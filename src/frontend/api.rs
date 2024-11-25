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
        SuccessResponse,
    },
    frontend::error::MyResult,
};
use anyhow::anyhow;
use http::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::LazyLock};
use url::Url;

pub static CLIENT: LazyLock<ApiClient> = LazyLock::new(|| {
    #[cfg(feature = "ssr")]
    {
        ApiClient::new(reqwest::Client::new(), None)
    }
    #[cfg(not(feature = "ssr"))]
    {
        ApiClient::new()
    }
});

#[derive(Clone)]
pub struct ApiClient {
    #[cfg(feature = "ssr")]
    client: reqwest::Client,
    pub hostname: String,
    ssl: bool,
}

impl ApiClient {
    #[cfg(feature = "ssr")]
    pub fn new(client: reqwest::Client, hostname_: Option<String>) -> Self {
        use leptos::config::get_config_from_str;
        let leptos_options = get_config_from_str(include_str!("../../Cargo.toml")).unwrap();
        let mut hostname = leptos_options.site_addr.to_string();
        // required for tests
        if let Some(hostname_) = hostname_ {
            hostname = hostname_;
        }
        Self {
            client,
            hostname,
            ssl: false,
        }
    }
    #[cfg(not(feature = "ssr"))]
    pub fn new() -> Self {
        use leptos_use::use_document;
        let hostname = use_document().location().unwrap().host().unwrap();
        let ssl = !cfg!(debug_assertions);
        Self { hostname, ssl }
    }

    pub async fn get_article(&self, data: GetArticleForm) -> MyResult<ArticleView> {
        self.get("/api/v1/article", Some(data)).await
    }

    pub async fn list_articles(&self, data: ListArticlesForm) -> MyResult<Vec<DbArticle>> {
        self.get("/api/v1/article/list", Some(data)).await
    }

    pub async fn register(&self, register_form: RegisterUserForm) -> MyResult<LocalUserView> {
        self.post("/api/v1/account/register", Some(&register_form))
            .await
    }

    pub async fn login(&self, login_form: LoginUserForm) -> MyResult<LocalUserView> {
        self.post("/api/v1/account/login", Some(&login_form)).await
    }

    pub async fn create_article(&self, data: &CreateArticleForm) -> MyResult<ArticleView> {
        self.send(Method::POST, "/api/v1/article", Some(&data))
            .await
    }

    pub async fn edit_article_with_conflict(
        &self,
        edit_form: &EditArticleForm,
    ) -> MyResult<Option<ApiConflict>> {
        self.send(Method::PATCH, "/api/v1/article", Some(&edit_form))
            .await
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
        self.get("/api/v1/user/notifications/list", None::<()>)
            .await
    }

    pub async fn notifications_count(&self) -> MyResult<usize> {
        self.get("/api/v1/user/notifications/count", None::<()>)
            .await
    }

    pub async fn approve_article(&self, article_id: ArticleId, approve: bool) -> MyResult<()> {
        let form = ApproveArticleForm {
            article_id,
            approve,
        };
        self.post("/api/v1/article/approve", Some(&form)).await
    }

    pub async fn delete_conflict(&self, conflict_id: ConflictId) -> MyResult<()> {
        let form = DeleteConflictForm { conflict_id };
        self.send(Method::DELETE, "/api/v1/conflict", Some(form))
            .await
    }

    pub async fn search(&self, search_form: &SearchArticleForm) -> MyResult<Vec<DbArticle>> {
        self.get("/api/v1/search", Some(search_form)).await
    }

    pub async fn get_local_instance(&self) -> MyResult<InstanceView> {
        self.get("/api/v1/instance", None::<i32>).await
    }

    pub async fn get_instance(&self, get_form: &GetInstance) -> MyResult<InstanceView> {
        self.get("/api/v1/instance", Some(&get_form)).await
    }

    pub async fn list_instances(&self) -> MyResult<Vec<DbInstance>> {
        self.get("/api/v1/instance/list", None::<i32>).await
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
            .get("/api/v1/instance/resolve", Some(resolve_form))
            .await?;

        // send follow
        let follow_form = FollowInstance {
            id: instance_resolved.id,
        };
        self.follow_instance(follow_form).await?;
        Ok(instance_resolved)
    }

    pub async fn follow_instance(&self, follow_form: FollowInstance) -> MyResult<SuccessResponse> {
        self.post("/api/v1/instance/follow", Some(follow_form))
            .await
    }

    pub async fn site(&self) -> MyResult<SiteView> {
        self.get("/api/v1/site", None::<()>).await
    }

    pub async fn logout(&self) -> MyResult<SuccessResponse> {
        self.post("/api/v1/account/logout", None::<()>).await
    }

    pub async fn fork_article(&self, form: &ForkArticleForm) -> MyResult<ArticleView> {
        Ok(self.post("/api/v1/article/fork", Some(form)).await.unwrap())
    }

    pub async fn protect_article(&self, params: &ProtectArticleForm) -> MyResult<DbArticle> {
        self.post("/api/v1/article/protect", Some(params)).await
    }

    pub async fn resolve_article(&self, id: Url) -> MyResult<ArticleView> {
        let resolve_object = ResolveObject { id };
        self.get("/api/v1/article/resolve", Some(resolve_object))
            .await
    }

    pub async fn resolve_instance(&self, id: Url) -> MyResult<DbInstance> {
        let resolve_object = ResolveObject { id };
        self.get("/api/v1/instance/resolve", Some(resolve_object))
            .await
    }
    pub async fn get_user(&self, data: GetUserForm) -> MyResult<DbPerson> {
        self.get("/api/v1/user", Some(data)).await
    }

    async fn get<T, R>(&self, endpoint: &str, query: Option<R>) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        self.send(Method::GET, endpoint, query).await
    }

    async fn post<T, R>(&self, endpoint: &str, query: Option<R>) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        self.send(Method::POST, endpoint, query).await
    }

    #[cfg(feature = "ssr")]
    async fn send<P, T>(&self, method: Method, path: &str, params: Option<P>) -> MyResult<T>
    where
        P: Serialize + Debug,
        T: for<'de> Deserialize<'de>,
    {
        use crate::common::{Auth, AUTH_COOKIE};
        use leptos::prelude::use_context;
        use reqwest::header::HeaderName;
        let mut req = self
            .client
            .request(method.clone(), self.request_endpoint(path));
        req = if method == Method::GET {
            req.query(&params)
        } else {
            req.form(&params)
        };
        let auth = use_context::<Auth>();
        if let Some(Auth(Some(auth))) = auth {
            req = req.header(HeaderName::from_static(AUTH_COOKIE), auth);
        }
        let res = req.send().await?;
        let status = res.status();
        let text = res.text().await?.to_string();
        Self::response(status.into(), text)
    }

    #[cfg(not(feature = "ssr"))]
    fn send<'a, P, T>(
        &'a self,
        method: Method,
        path: &'a str,
        params: Option<P>,
    ) -> impl std::future::Future<Output = MyResult<T>> + Send + 'a
    where
        P: Serialize + Debug + 'a,
        T: for<'de> Deserialize<'de>,
    {
        use gloo_net::http::*;
        use leptos::prelude::on_cleanup;
        use send_wrapper::SendWrapper;
        use web_sys::RequestCredentials;

        SendWrapper::new(async move {
            let abort_controller = SendWrapper::new(web_sys::AbortController::new().ok());
            let abort_signal = abort_controller.as_ref().map(|a| a.signal());

            // abort in-flight requests if, e.g., we've navigated away from this page
            on_cleanup(move || {
                if let Some(abort_controller) = abort_controller.take() {
                    abort_controller.abort()
                }
            });

            let path_with_endpoint = self.request_endpoint(path);
            let params_encoded = serde_urlencoded::to_string(&params).unwrap();
            let path = if method == Method::GET {
                // Cannot pass the form data directly but need to convert it manually
                // https://github.com/rustwasm/gloo/issues/378
                format!("{path_with_endpoint}?{params_encoded}")
            } else {
                path_with_endpoint
            };

            let builder = RequestBuilder::new(&path)
                .method(method.clone())
                .abort_signal(abort_signal.as_ref())
                .credentials(RequestCredentials::Include);
            let req = if method != Method::GET {
                builder
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(params_encoded)
            } else {
                builder.build()
            }
            .unwrap();
            let res = req.send().await.unwrap();
            let status = res.status();
            let text = res.text().await.unwrap();
            Self::response(status, text)
        })
    }

    fn response<T>(status: u16, text: String) -> MyResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        if status == StatusCode::OK {
            Ok(serde_json::from_str(&text).map_err(|e| anyhow!("Json error on {text}: {e}"))?)
        } else {
            Err(anyhow!("API error: {text}").into())
        }
    }

    fn request_endpoint(&self, path: &str) -> String {
        let protocol = if self.ssl { "https" } else { "http" };
        format!("{protocol}://{}{path}", &self.hostname)
    }
}
