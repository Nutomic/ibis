use crate::common::ApiConflict;
use crate::common::ResolveObject;
use crate::common::{ArticleView, LoginUserData, RegisterUserData};
use crate::common::{CreateArticleData, EditArticleData, ForkArticleData, LocalUserView};
use crate::common::{DbArticle, GetArticleData};
use crate::common::{DbInstance, FollowInstance, InstanceView, SearchArticleData};
use crate::frontend::error::MyResult;
use anyhow::anyhow;
use once_cell::sync::Lazy;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use url::Url;

pub static CLIENT: Lazy<Client> = Lazy::new(Client::new);

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    pub hostname: String,
}

impl ApiClient {
    pub fn new(client: Client, hostname: String) -> Self {
        Self { client, hostname }
    }

    async fn get_query<T, R>(&self, endpoint: &str, query: Option<R>) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize,
    {
        let mut req = self
            .client
            .get(format!("http://{}/api/v1/{}", &self.hostname, endpoint));
        if let Some(query) = query {
            req = req.query(&query);
        }
        handle_json_res::<T>(req).await
    }

    pub async fn get_article(&self, data: GetArticleData) -> MyResult<ArticleView> {
        self.get_query::<ArticleView, _>("article", Some(data))
            .await
    }

    pub async fn register(&self, register_form: RegisterUserData) -> MyResult<LocalUserView> {
        let req = self
            .client
            .post(format!("http://{}/api/v1/account/register", self.hostname))
            .form(&register_form);
        handle_json_res::<LocalUserView>(req).await
    }

    pub async fn login(&self, login_form: LoginUserData) -> MyResult<LocalUserView> {
        let req = self
            .client
            .post(format!("http://{}/api/v1/account/login", self.hostname))
            .form(&login_form);
        handle_json_res::<LocalUserView>(req).await
    }

    pub async fn create_article(&self, title: String, new_text: String) -> MyResult<ArticleView> {
        let create_form = CreateArticleData {
            title: title.clone(),
        };
        let req = self
            .client
            .post(format!("http://{}/api/v1/article", &self.hostname))
            .form(&create_form);
        let article: ArticleView = handle_json_res(req).await?;

        // create initial edit to ensure that conflicts are generated (there are no conflicts on empty file)
        // TODO: maybe take initial text directly in create article, no reason to have empty article
        let edit_form = EditArticleData {
            article_id: article.article.id,
            new_text,
            previous_version_id: article.latest_version,
            resolve_conflict_id: None,
        };
        Ok(self.edit_article(&edit_form).await.unwrap())
    }

    pub async fn edit_article_with_conflict(
        &self,
        edit_form: &EditArticleData,
    ) -> MyResult<Option<ApiConflict>> {
        let req = self
            .client
            .patch(format!("http://{}/api/v1/article", self.hostname))
            .form(edit_form);
        handle_json_res(req).await
    }

    pub async fn edit_article(&self, edit_form: &EditArticleData) -> MyResult<ArticleView> {
        let edit_res = self.edit_article_with_conflict(edit_form).await?;
        assert!(edit_res.is_none());

        self.get_article(GetArticleData {
            title: None,
            instance_id: None,
            id: Some(edit_form.article_id),
        })
        .await
    }

    pub async fn search(&self, search_form: &SearchArticleData) -> MyResult<Vec<DbArticle>> {
        self.get_query("search", Some(search_form)).await
    }

    pub async fn get_local_instance(&self) -> MyResult<InstanceView> {
        self.get_query("instance", None::<i32>).await
    }

    pub async fn follow_instance(&self, follow_instance: &str) -> MyResult<DbInstance> {
        // fetch beta instance on alpha
        let resolve_form = ResolveObject {
            id: Url::parse(&format!("http://{}", follow_instance))?,
        };
        let instance_resolved: DbInstance =
            get_query(&self.hostname, "instance/resolve", Some(resolve_form)).await?;

        // send follow
        let follow_form = FollowInstance {
            id: instance_resolved.id,
        };
        // cant use post helper because follow doesnt return json
        let res = self
            .client
            .post(format!("http://{}/api/v1/instance/follow", self.hostname))
            .form(&follow_form)
            .send()
            .await?;
        if res.status() == StatusCode::OK {
            Ok(instance_resolved)
        } else {
            Err(anyhow!("API error: {}", res.text().await?).into())
        }
    }

    pub async fn my_profile(&self) -> MyResult<LocalUserView> {
        let req = self.client.get(format!(
            "http://{}/api/v1/account/my_profile",
            self.hostname
        ));
        handle_json_res::<LocalUserView>(req).await
    }

    pub async fn logout(&self) -> MyResult<()> {
        self.client
            .get(format!("http://{}/api/v1/account/logout", self.hostname))
            .send()
            .await?;
        Ok(())
    }

    pub async fn fork_article(&self, form: &ForkArticleData) -> MyResult<ArticleView> {
        let req = self
            .client
            .post(format!("http://{}/api/v1/article/fork", self.hostname))
            .form(form);
        Ok(handle_json_res(req).await.unwrap())
    }

    pub async fn get_conflicts(&self) -> MyResult<Vec<ApiConflict>> {
        let req = self
            .client
            .get(format!("http://{}/api/v1/edit_conflicts", &self.hostname));
        Ok(handle_json_res(req).await.unwrap())
    }

    pub async fn resolve_article(&self, id: Url) -> MyResult<ArticleView> {
        let resolve_object = ResolveObject { id };
        get_query(&self.hostname, "article/resolve", Some(resolve_object)).await
    }

    pub async fn resolve_instance(&self, id: Url) -> MyResult<DbInstance> {
        let resolve_object = ResolveObject { id };
        get_query(&self.hostname, "instance/resolve", Some(resolve_object)).await
    }
}

async fn get_query<T, R>(hostname: &str, endpoint: &str, query: Option<R>) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
    R: Serialize,
{
    let mut req = CLIENT.get(format!("http://{}/api/v1/{}", hostname, endpoint));
    if let Some(query) = query {
        req = req.query(&query);
    }
    handle_json_res::<T>(req).await
}

async fn handle_json_res<T>(req: RequestBuilder) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    let res = req.send().await?;
    let status = res.status();
    let text = res.text().await?;
    if status == reqwest::StatusCode::OK {
        Ok(serde_json::from_str(&text).map_err(|e| anyhow!("Json error on {text}: {e}"))?)
    } else {
        Err(anyhow!("API error: {text}").into())
    }
}
