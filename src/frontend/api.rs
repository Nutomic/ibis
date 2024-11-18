use crate::{
    common::{
        newtypes::{ArticleId, ConflictId},
        utils::http_protocol_str,
        ApiConflict,
        ApproveArticleForm,
        ArticleView,
        CreateArticleForm,
        DbArticle,
        DbInstance,
        DbPerson,
        DeleteConflictForm,
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
use send_wrapper::SendWrapper;
use serde::{Deserialize, Serialize};
use std::{future::Future, sync::LazyLock};
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
            use leptos_use::use_document;
            hostname = use_document().location().unwrap().host().unwrap();
            ssl = !cfg!(debug_assertions);
        }
        #[cfg(feature = "ssr")]
        {
            use leptos::config::get_config_from_str;
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

    pub fn get_article(
        &self,
        data: GetArticleForm,
    ) -> impl Future<Output = MyResult<ArticleView>> + Send + '_ {
        SendWrapper::new(async move { self.get_query("/api/v1/article", Some(data)).await })
    }

    pub fn list_articles(
        &self,
        data: ListArticlesForm,
    ) -> impl Future<Output = MyResult<Vec<DbArticle>>> + Send + '_ {
        SendWrapper::new(async move { self.get_query("/api/v1/article/list", Some(data)).await })
    }

    pub fn register(
        &self,
        register_form: RegisterUserForm,
    ) -> impl Future<Output = MyResult<LocalUserView>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .post(self.request_endpoint("/api/v1/account/register"))
                .form(&register_form);
            handle_json_res::<LocalUserView>(req).await
        })
    }

    pub fn login(
        &self,
        login_form: LoginUserForm,
    ) -> impl Future<Output = MyResult<LocalUserView>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .post(self.request_endpoint("/api/v1/account/login"))
                .form(&login_form);
            handle_json_res::<LocalUserView>(req).await
        })
    }

    pub fn create_article(
        &self,
        data: CreateArticleForm,
    ) -> impl Future<Output = MyResult<ArticleView>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .post(self.request_endpoint("/api/v1/article"))
                .form(&data);
            handle_json_res(req).await
        })
    }

    pub fn edit_article_with_conflict(
        &self,
        edit_form: EditArticleForm,
    ) -> impl Future<Output = MyResult<Option<ApiConflict>>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .patch(self.request_endpoint("/api/v1/article"))
                .form(&edit_form);
            handle_json_res(req).await
        })
    }

    pub fn edit_article(
        &self,
        edit_form: EditArticleForm,
    ) -> impl Future<Output = MyResult<ArticleView>> + Send + '_ {
        SendWrapper::new(async move {
            let article_id = edit_form.article_id;
            let edit_res = self.edit_article_with_conflict(edit_form).await?;
            assert!(edit_res.is_none());

            self.get_article(GetArticleForm {
                title: None,
                domain: None,
                id: Some(article_id),
            })
            .await
        })
    }

    pub fn notifications_list(
        &self,
    ) -> impl Future<Output = MyResult<Vec<Notification>>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .get(self.request_endpoint("/api/v1/user/notifications/list"));
            handle_json_res(req).await
        })
    }

    pub fn notifications_count(&self) -> impl Future<Output = MyResult<usize>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .get(self.request_endpoint("/api/v1/user/notifications/count"));
            handle_json_res(req).await
        })
    }

    pub fn approve_article(
        &self,
        article_id: ArticleId,
        approve: bool,
    ) -> impl Future<Output = MyResult<()>> + Send + '_ {
        SendWrapper::new(async move {
            let form = ApproveArticleForm {
                article_id,
                approve,
            };
            let req = self
                .client
                .post(self.request_endpoint("/api/v1/article/approve"))
                .form(&form);
            handle_json_res(req).await
        })
    }

    pub fn delete_conflict(
        &self,
        conflict_id: ConflictId,
    ) -> impl Future<Output = MyResult<()>> + Send + '_ {
        SendWrapper::new(async move {
            let form = DeleteConflictForm { conflict_id };
            let req = self
                .client
                .delete(self.request_endpoint("/api/v1/conflict"))
                .form(&form);
            handle_json_res(req).await
        })
    }

    pub fn search(
        &self,
        search_form: SearchArticleForm,
    ) -> impl Future<Output = MyResult<Vec<DbArticle>>> + Send + '_ {
        SendWrapper::new(async move { self.get_query("/api/v1/search", Some(search_form)).await })
    }

    pub fn get_local_instance(
        &self,
    ) -> impl Future<Output = MyResult<Vec<InstanceView>>> + Send + '_ {
        SendWrapper::new(async move { self.get_query("/api/v1/instance", None::<i32>).await })
    }

    pub fn get_instance(
        &self,
        get_form: GetInstance,
    ) -> impl Future<Output = MyResult<InstanceView>> + Send + '_ {
        SendWrapper::new(async move { self.get_query("/api/v1/instance", Some(get_form)).await })
    }

    pub fn list_instances(&self) -> impl Future<Output = MyResult<Vec<DbInstance>>> + Send + '_ {
        SendWrapper::new(async move { self.get_query("/api/v1/instance/list", None::<i32>).await })
    }

    pub fn follow_instance_with_resolve(
        &self,
        follow_instance: String,
    ) -> impl Future<Output = MyResult<DbInstance>> + Send + '_ {
        SendWrapper::new(async move {
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
        })
    }

    pub fn follow_instance(
        &self,
        follow_form: FollowInstance,
    ) -> impl Future<Output = MyResult<()>> + Send + '_ {
        SendWrapper::new(async move {
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
        })
    }

    pub fn site(&self) -> impl Future<Output = MyResult<SiteView>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self.client.get(self.request_endpoint("/api/v1/site"));
            handle_json_res(req).await
        })
    }

    pub fn logout(&self) -> impl Future<Output = MyResult<()>> + Send + '_ {
        SendWrapper::new(async move {
            self.client
                .get(self.request_endpoint("/api/v1/account/logout"))
                .send()
                .await?;
            Ok(())
        })
    }

    pub fn fork_article(
        &self,
        form: ForkArticleForm,
    ) -> impl Future<Output = MyResult<ArticleView>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .post(self.request_endpoint("/api/v1/article/fork"))
                .form(&form);
            Ok(handle_json_res(req).await.unwrap())
        })
    }

    pub fn protect_article(
        &self,
        params: ProtectArticleForm,
    ) -> impl Future<Output = MyResult<DbArticle>> + Send + '_ {
        SendWrapper::new(async move {
            let req = self
                .client
                .post(self.request_endpoint("/api/v1/article/protect"))
                .form(&params);
            handle_json_res(req).await
        })
    }

    pub fn resolve_article(
        &self,
        id: Url,
    ) -> impl Future<Output = MyResult<ArticleView>> + Send + '_ {
        SendWrapper::new(async move {
            let resolve_object = ResolveObject { id };
            self.get_query("/api/v1/article/resolve", Some(resolve_object))
                .await
        })
    }

    pub fn resolve_instance(
        &self,
        id: Url,
    ) -> impl Future<Output = MyResult<DbInstance>> + Send + '_ {
        SendWrapper::new(async move {
            let resolve_object = ResolveObject { id };
            self.get_query("/api/v1/instance/resolve", Some(resolve_object))
                .await
        })
    }
    pub fn get_user(
        &self,
        data: GetUserForm,
    ) -> impl Future<Output = MyResult<DbPerson>> + Send + '_ {
        SendWrapper::new(async move { self.get_query("/api/v1/user", Some(data)).await })
    }

    fn request_endpoint(&self, path: &str) -> String {
        let protocol = if self.ssl { "https" } else { "http" };
        format!("{protocol}://{}{path}", &self.hostname)
    }
}

async fn handle_json_res<T>(mut req: RequestBuilder) -> MyResult<T>
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
        use leptos::prelude::use_context;
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
