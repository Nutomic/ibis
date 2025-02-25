use anyhow::anyhow;
use axum::{
    body::Body,
    extract::{Request, State},
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use http::{HeaderMap, HeaderName, HeaderValue};
use ibis_database::error::BackendResult;
use include_dir::include_dir;
use leptos::prelude::*;
use mime_guess::mime::APPLICATION_OCTET_STREAM;
use tower::util::ServiceExt;
use tower_http::services::ServeDir;

// from https://github.com/leptos-rs/start-axum

#[debug_handler]
pub async fn file_and_error_handler(
    State(options): State<LeptosOptions>,
    request: Request<Body>,
) -> BackendResult<Response<Body>> {
    if cfg!(debug_assertions) {
        // in debug mode serve assets directly from local folder
        Ok(ServeDir::new(options.site_root.as_ref())
            .oneshot(request)
            .await
            .into_response())
    } else {
        // for production embed assets in binary
        let mut headers = HeaderMap::new();
        let dir = include_dir!("target/site/");
        let path = request.uri().path().replacen('/', "", 1);
        let content = dir.get_file(&path).ok_or(anyhow!("not found"))?;
        headers.insert(
            HeaderName::from_static("content-type"),
            HeaderValue::from_str(
                mime_guess::from_path(path)
                    .first_raw()
                    .unwrap_or_else(|| APPLICATION_OCTET_STREAM.essence_str()),
            )?,
        );
        headers.insert(
            HeaderName::from_static("cache-control"),
            HeaderValue::from_static("max-age=3600, public"),
        );
        Ok((headers, content.contents()).into_response())
    }
}
