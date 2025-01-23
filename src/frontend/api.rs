use crate::common::{
    article::*,
    instance::*,
    newtypes::{ArticleId, ConflictId, PersonId},
    user::*,
    utils::http_protocol_str,
    *,
};
use comment::{CreateCommentParams, DbCommentView, EditCommentParams};
use http::{Method, StatusCode};
use leptos::{prelude::ServerFnError, server_fn::error::NoCustomError};
use log::{error, info};
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

#[derive(Clone, Debug)]
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

    pub async fn get_article(&self, data: GetArticleParams) -> Option<DbArticleView> {
        self.get("/api/v1/article", Some(data)).await
    }

    pub async fn list_articles(&self, data: ListArticlesParams) -> Option<Vec<DbArticle>> {
        Some(self.get("/api/v1/article/list", Some(data)).await.unwrap())
    }

    pub async fn register(
        &self,
        params: RegisterUserParams,
    ) -> Result<LocalUserView, ServerFnError> {
        self.post("/api/v1/account/register", Some(&params)).await
    }

    pub async fn login(&self, params: LoginUserParams) -> Result<LocalUserView, ServerFnError> {
        self.post("/api/v1/account/login", Some(&params)).await
    }

    pub async fn create_article(
        &self,
        data: &CreateArticleParams,
    ) -> Result<DbArticleView, ServerFnError> {
        self.post("/api/v1/article", Some(&data)).await
    }

    pub async fn edit_article_with_conflict(
        &self,
        params: &EditArticleParams,
    ) -> Result<Option<ApiConflict>, ServerFnError> {
        self.patch("/api/v1/article", Some(&params)).await
    }

    #[cfg(debug_assertions)]
    pub async fn edit_article(&self, params: &EditArticleParams) -> Option<DbArticleView> {
        let edit_res = self
            .edit_article_with_conflict(params)
            .await
            .map_err(|e| error!("edit failed {e}"))
            .ok()?;
        assert_eq!(None, edit_res);

        self.get_article(GetArticleParams {
            title: None,
            domain: None,
            id: Some(params.article_id),
        })
        .await
    }

    pub async fn create_comment(
        &self,
        params: &CreateCommentParams,
    ) -> Result<DbCommentView, ServerFnError> {
        self.post("/api/v1/comment", Some(&params)).await
    }

    pub async fn edit_comment(
        &self,
        params: &EditCommentParams,
    ) -> Result<DbCommentView, ServerFnError> {
        self.patch("/api/v1/comment", Some(&params)).await
    }

    pub async fn notifications_list(&self) -> Option<Vec<Notification>> {
        self.get("/api/v1/user/notifications/list", None::<()>)
            .await
    }

    pub async fn notifications_count(&self) -> Option<usize> {
        self.get("/api/v1/user/notifications/count", None::<()>)
            .await
    }

    pub async fn approve_article(&self, article_id: ArticleId, approve: bool) -> Option<()> {
        let params = ApproveArticleParams {
            article_id,
            approve,
        };
        result_to_option(self.post("/api/v1/article/approve", Some(&params)).await)
    }

    pub async fn delete_conflict(&self, conflict_id: ConflictId) -> Option<()> {
        let params = DeleteConflictParams { conflict_id };
        result_to_option(
            self.send(Method::DELETE, "/api/v1/conflict", Some(params))
                .await,
        )
    }

    pub async fn search(
        &self,
        params: &SearchArticleParams,
    ) -> Result<Vec<DbArticle>, ServerFnError> {
        self.send(Method::GET, "/api/v1/search", Some(params)).await
    }

    pub async fn get_local_instance(&self) -> Option<InstanceView> {
        self.get("/api/v1/instance", None::<i32>).await
    }

    pub async fn get_instance(&self, params: &GetInstanceParams) -> Option<InstanceView> {
        self.get("/api/v1/instance", Some(&params)).await
    }

    pub async fn list_instances(&self) -> Option<Vec<DbInstance>> {
        self.get("/api/v1/instance/list", None::<i32>).await
    }

    pub async fn update_local_instance(
        &self,
        params: &UpdateInstanceParams,
    ) -> Result<DbInstance, ServerFnError> {
        self.patch("/api/v1/instance", Some(params)).await
    }

    pub async fn follow_instance_with_resolve(&self, follow_instance: &str) -> Option<DbInstance> {
        // fetch beta instance on alpha
        let params = ResolveObjectParams {
            id: Url::parse(&format!("{}://{}", http_protocol_str(), follow_instance))
                .map_err(|e| error!("invalid url {e}"))
                .ok()?,
        };
        let instance_resolved: DbInstance =
            self.get("/api/v1/instance/resolve", Some(params)).await?;

        // send follow
        let params = FollowInstanceParams {
            id: instance_resolved.id,
        };
        self.follow_instance(params).await?;
        Some(instance_resolved)
    }

    pub async fn follow_instance(&self, params: FollowInstanceParams) -> Option<SuccessResponse> {
        result_to_option(self.post("/api/v1/instance/follow", Some(params)).await)
    }

    pub async fn site(&self) -> Option<SiteView> {
        self.get("/api/v1/site", None::<()>).await
    }

    pub async fn logout(&self) -> Option<SuccessResponse> {
        result_to_option(self.post("/api/v1/account/logout", None::<()>).await)
    }

    pub async fn fork_article(
        &self,
        params: &ForkArticleParams,
    ) -> Result<DbArticleView, ServerFnError> {
        self.post("/api/v1/article/fork", Some(params)).await
    }

    pub async fn protect_article(
        &self,
        params: &ProtectArticleParams,
    ) -> Result<DbArticle, ServerFnError> {
        self.post("/api/v1/article/protect", Some(params)).await
    }

    pub async fn resolve_article(&self, id: Url) -> Result<DbArticleView, ServerFnError> {
        let resolve_object = ResolveObjectParams { id };
        self.send(Method::GET, "/api/v1/article/resolve", Some(resolve_object))
            .await
    }

    pub async fn resolve_instance(&self, id: Url) -> Result<DbInstance, ServerFnError> {
        let resolve_object = ResolveObjectParams { id };
        self.send(
            Method::GET,
            "/api/v1/instance/resolve",
            Some(resolve_object),
        )
        .await
    }

    pub async fn get_user(&self, data: GetUserParams) -> Option<DbPerson> {
        self.get("/api/v1/user", Some(data)).await
    }

    pub async fn update_user_profile(
        &self,
        data: UpdateUserParams,
    ) -> Result<SuccessResponse, ServerFnError> {
        self.post("/api/v1/account/update", Some(data)).await
    }

    pub async fn get_article_edits(&self, article_id: ArticleId) -> Option<Vec<EditView>> {
        let data = GetEditList {
            article_id: Some(article_id),
            ..Default::default()
        };
        self.get("/api/v1/edit/list", Some(data)).await
    }

    pub async fn get_person_edits(&self, person_id: PersonId) -> Option<Vec<EditView>> {
        let data = GetEditList {
            person_id: Some(person_id),
            ..Default::default()
        };
        self.get("/api/v1/edit/list", Some(data)).await
    }

    async fn get<T, R>(&self, endpoint: &str, query: Option<R>) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        result_to_option(self.send(Method::GET, endpoint, query).await)
    }

    async fn post<T, R>(&self, endpoint: &str, query: Option<R>) -> Result<T, ServerFnError>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        self.send(Method::POST, endpoint, query).await
    }

    async fn patch<T, R>(&self, endpoint: &str, query: Option<R>) -> Result<T, ServerFnError>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        self.send(Method::PATCH, endpoint, query).await
    }

    #[cfg(feature = "ssr")]
    async fn send<P, T>(
        &self,
        method: Method,
        path: &str,
        params: Option<P>,
    ) -> Result<T, ServerFnError>
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
        let url = res.url().to_string();
        let text = res.text().await?.to_string();
        Self::response(status.into(), text, &url)
    }

    #[cfg(not(feature = "ssr"))]
    fn send<'a, P, T>(
        &'a self,
        method: Method,
        path: &'a str,
        params: Option<P>,
    ) -> impl std::future::Future<Output = Result<T, ServerFnError>> + Send + 'a
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
            let res = req.send().await?;
            let status = res.status();
            let text = res.text().await?;
            Self::response(status, text, &res.url())
        })
    }

    fn response<T>(status: u16, text: String, url: &str) -> Result<T, ServerFnError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json = serde_json::from_str(&text).map_err(|e| {
            info!("Failed to deserialize api response: {e} from {text} on {url}");
            ServerFnError::<NoCustomError>::Deserialization(text.clone())
        })?;
        if status == StatusCode::OK {
            Ok(json)
        } else {
            info!("API error: {text} on {url} status {status}");
            Err(ServerFnError::Response(text))
        }
    }

    fn request_endpoint(&self, path: &str) -> String {
        let protocol = if self.ssl { "https" } else { "http" };
        format!("{protocol}://{}{path}", &self.hostname)
    }
}

fn result_to_option<T>(val: Result<T, ServerFnError>) -> Option<T> {
    match val {
        Ok(v) => Some(v),
        Err(e) => {
            error!("API error: {e}");
            None
        }
    }
}
