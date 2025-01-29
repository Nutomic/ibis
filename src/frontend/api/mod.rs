use crate::frontend::utils::errors::{FrontendError, FrontendResult};
use http::{Method, StatusCode};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, sync::LazyLock};

pub mod article;
pub mod comment;
pub mod instance;
pub mod user;

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
        let leptos_options = get_config_from_str(include_str!("../../../Cargo.toml")).unwrap();
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

    async fn get<T, R>(&self, endpoint: &str, query: Option<R>) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        result_to_option(self.send(Method::GET, endpoint, query).await)
    }

    async fn post<T, R>(&self, endpoint: &str, query: Option<R>) -> FrontendResult<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        self.send(Method::POST, endpoint, query).await
    }

    async fn patch<T, R>(&self, endpoint: &str, query: Option<R>) -> FrontendResult<T>
    where
        T: for<'de> Deserialize<'de>,
        R: Serialize + Debug,
    {
        self.send(Method::PATCH, endpoint, query).await
    }

    #[cfg(feature = "ssr")]
    async fn send<P, T>(&self, method: Method, path: &str, params: Option<P>) -> FrontendResult<T>
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
    ) -> impl std::future::Future<Output = FrontendResult<T>> + Send + 'a
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

    fn response<T>(status: u16, text: String, url: &str) -> FrontendResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let json = serde_json::from_str(&text).map_err(|e| {
            info!("Failed to deserialize api response: {e} from {text} on {url}");
            FrontendError(text.clone())
        })?;
        if status == StatusCode::OK {
            Ok(json)
        } else {
            info!("API error: {text} on {url} status {status}");
            Err(FrontendError(text))
        }
    }

    fn request_endpoint(&self, path: &str) -> String {
        let protocol = if self.ssl { "https" } else { "http" };
        format!("{protocol}://{}{path}", &self.hostname)
    }
}

fn result_to_option<T>(val: FrontendResult<T>) -> Option<T> {
    match val {
        Ok(v) => Some(v),
        Err(e) => {
            error!("API error: {e}");
            None
        }
    }
}
