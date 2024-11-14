use crate::{
    common::{
        newtypes::ArticleId,
        utils::http_protocol_str,
        ApiConflict,
        ApproveArticleForm,
        ArticleView,
        CreateArticleForm,
        DbArticle,
        DbInstance,
        DbPerson,
        EditArticleForm,
        FollowInstance,
        ForkArticleForm,
        GetArticleForm,
        GetInstance,
        GetUserForm,
        InstanceView,
        ListArticlesForm,
        LocalUserView,
        LoginUserForm,
        Notification,
        ProtectArticleForm,
        RegisterUserForm,
        ResolveObject,
        SearchArticleForm,
        SiteView,
    },
    frontend::error::MyResult,
};
use anyhow::anyhow;
use reqwest::{Client, RequestBuilder, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use url::Url;

pub static CLIENT: LazyLock<ApiClient> = LazyLock::new(|| ApiClient::new(Client::new(), None));

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    pub hostname: String,
    ssl: bool,
}

impl ApiClient {
    pub fn new(client: Client, hostname_: Option<String>) -> Self {
        let mut hostname;
        let ssl;
        #[cfg(not(feature = "ssr"))]
        {
            hostname = web_sys::window().unwrap().location().host().unwrap();
            ssl = !cfg!(debug_assertions);
        }
        #[cfg(feature = "ssr")]
        {
            use leptos::leptos_config::get_config_from_str;
            let leptos_options = get_config_from_str(include_str!("../../Cargo.toml")).unwrap();
            hostname = leptos_options.site_addr.to_string();
            ssl = false;
        }
        // required for tests
        if let Some(hostname_) = hostname_ {
            hostname = hostname_;
        }
        Self {
            client,
            hostname,
            ssl,
        }
    }

    async fn get_query<T, R>(&self, endpoint: &str, query: Option<R>) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize,
    {
        let mut req = self.client.get(self.request_endpoint(endpoint));
        if let Some(query) = query {
            req = req.query(&query);
        }
        handle_json_res::<T>(req).await
    }

    pub async fn get_article(&self, data: GetArticleForm) -> MyResult<ArticleView> {
        self.get_query("/api/v1/article", Some(data)).await
    }

    pub async fn list_articles(&self, data: ListArticlesForm) -> MyResult<Vec<DbArticle>> {
        self.get_query("/api/v1/article/list", Some(data)).await
    }

    pub async fn register(&self, register_form: RegisterUserForm) -> MyResult<LocalUserView> {
        let req = self
            .client
            .post(self.request_endpoint("/api/v1/account/register"))
            .form(&register_form);
        handle_json_res::<LocalUserView>(req).await
    }

    pub async fn login(&self, login_form: LoginUserForm) -> MyResult<LocalUserView> {
        let req = self
            .client
            .post(self.request_endpoint("/api/v1/account/login"))
            .form(&login_form);
        handle_json_res::<LocalUserView>(req).await
    }

    pub async fn create_article(&self, data: &CreateArticleForm) -> MyResult<ArticleView> {
        let req = self
            .client
            .post(self.request_endpoint("/api/v1/article"))
            .form(data);
        handle_json_res(req).await
    }

    pub async fn edit_article_with_conflict(
        &self,
        edit_form: &EditArticleForm,
    ) -> MyResult<Option<ApiConflict>> {
        let req = self
            .client
            .patch(self.request_endpoint("/api/v1/article"))
            .form(edit_form);
        handle_json_res(req).await
    }

    pub async fn edit_article(&self, edit_form: &EditArticleForm) -> MyResult<ArticleView> {
        let edit_res = self.edit_article_with_conflict(edit_form).await?;
        assert!(edit_res.is_none());

        self.get_article(GetArticleForm {
            title: None,
            domain: None,
            id: Some(edit_form.article_id),
        })
        .await
    }

    pub async fn notifications_list(&self) -> MyResult<Vec<Notification>> {
        let req = self
            .client
            .get(self.request_endpoint("/api/v1/user/notifications/list"));
        handle_json_res(req).await
    }

    pub async fn notifications_count(&self) -> MyResult<usize> {
        let req = self
            .client
            .get(self.request_endpoint("/api/v1/user/notifications/count"));
        handle_json_res(req).await
    }

    pub async fn approve_article(&self, article_id: ArticleId) -> MyResult<DbArticle> {
        let form = ApproveArticleForm { article_id };
        let req = self
            .client
            .post(self.request_endpoint("/api/v1/article/approve"))
            .form(&form);
        handle_json_res(req).await
    }

    pub async fn search(&self, search_form: &SearchArticleForm) -> MyResult<Vec<DbArticle>> {
        self.get_query("/api/v1/search", Some(search_form)).await
    }

    pub async fn get_local_instance(&self) -> MyResult<InstanceView> {
        self.get_query("/api/v1/instance", None::<i32>).await
    }

    pub async fn get_instance(&self, get_form: &GetInstance) -> MyResult<InstanceView> {
        self.get_query("/api/v1/instance", Some(get_form)).await
    }

    pub async fn list_instances(&self) -> MyResult<Vec<DbInstance>> {
        self.get_query("/api/v1/instance/list", None::<i32>).await
    }

    pub async fn follow_instance_with_resolve(
        &self,
        follow_instance: &str,
    ) -> MyResult<DbInstance> {
        // fetch beta instance on alpha
        let resolve_form = ResolveObject {
            id: Url::parse(&format!("{}://{}", http_protocol_str(), follow_instance))?,
        };
        let instance_resolved: DbInstance = self
            .get_query("/api/v1/instance/resolve", Some(resolve_form))
            .await?;

        // send follow
        let follow_form = FollowInstance {
            id: instance_resolved.id,
        };
        self.follow_instance(follow_form).await?;
        Ok(instance_resolved)
    }

    pub async fn follow_instance(&self, follow_form: FollowInstance) -> MyResult<()> {
        // cant use post helper because follow doesnt return json
        let res = self
            .client
            .post(self.request_endpoint("/api/v1/instance/follow"))
            .form(&follow_form)
            .send()
            .await?;
        if res.status() == StatusCode::OK {
            Ok(())
        } else {
            Err(anyhow!("API error: {}", res.text().await?).into())
        }
    }

    pub async fn site(&self) -> MyResult<SiteView> {
        let req = self.client.get(self.request_endpoint("/api/v1/site"));
        handle_json_res(req).await
    }

    pub async fn logout(&self) -> MyResult<()> {
        self.client
            .get(self.request_endpoint("/api/v1/account/logout"))
            .send()
            .await?;
        Ok(())
    }

    pub async fn fork_article(&self, form: &ForkArticleForm) -> MyResult<ArticleView> {
        let req = self
            .client
            .post(self.request_endpoint("/api/v1/article/fork"))
            .form(form);
        Ok(handle_json_res(req).await.unwrap())
    }

    pub async fn protect_article(&self, params: &ProtectArticleForm) -> MyResult<DbArticle> {
        let req = self
            .client
            .post(self.request_endpoint("/api/v1/article/protect"))
            .form(params);
        handle_json_res(req).await
    }

    pub async fn resolve_article(&self, id: Url) -> MyResult<ArticleView> {
        let resolve_object = ResolveObject { id };
        self.get_query("/api/v1/article/resolve", Some(resolve_object))
            .await
    }

    pub async fn resolve_instance(&self, id: Url) -> MyResult<DbInstance> {
        let resolve_object = ResolveObject { id };
        self.get_query("/api/v1/instance/resolve", Some(resolve_object))
            .await
    }
    pub async fn get_user(&self, data: GetUserForm) -> MyResult<DbPerson> {
        self.get_query("/api/v1/user", Some(data)).await
    }

    fn request_endpoint(&self, path: &str) -> String {
        let protocol = if self.ssl { "https" } else { "http" };
        format!("{protocol}://{}{path}", &self.hostname)
    }
}

async fn handle_json_res<T>(#[allow(unused_mut)] mut req: RequestBuilder) -> MyResult<T>
where
    T: for<'de> Deserialize<'de>,
{
    #[cfg(not(feature = "ssr"))]
    {
        req = req.fetch_credentials_include();
    }

    #[cfg(feature = "ssr")]
    {
        use crate::common::{Auth, AUTH_COOKIE};
        use leptos::use_context;
        use reqwest::header::HeaderName;

        let auth = use_context::<Auth>();
        if let Some(Auth(Some(auth))) = auth {
            req = req.header(HeaderName::from_static(AUTH_COOKIE), auth);
        }
    }
    let res = req.send().await?;
    let status = res.status();
    let text = res.text().await?;
    if status == StatusCode::OK {
        Ok(serde_json::from_str(&text).map_err(|e| anyhow!("Json error on {text}: {e}"))?)
    } else {
        Err(anyhow!("API error: {text}").into())
    }
}
