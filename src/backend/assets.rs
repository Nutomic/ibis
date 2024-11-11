use axum::{
    body::Body,
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_macros::debug_handler;
use leptos::LeptosOptions;
use tower::ServiceExt;
use tower_http::services::ServeDir;

// from https://github.com/leptos-rs/start-axum

#[debug_handler]
pub async fn file_and_error_handler(
    State(options): State<LeptosOptions>,
    req: Request<Body>,
) -> Result<Response<Body>, StatusCode> {
    let root = options.site_root.clone();
    let (parts, _) = req.into_parts();

    let mut static_parts = parts.clone();
    static_parts.headers.clear();
    if let Some(encodings) = parts.headers.get("accept-encoding") {
        static_parts
            .headers
            .insert("accept-encoding", encodings.clone());
    }

    let res = get_static_file(Request::from_parts(static_parts, Body::empty()), &root).await?;

    if res.status() == StatusCode::OK {
        Ok(res.into_response())
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_static_file(request: Request<Body>, root: &str) -> Result<Response<Body>, StatusCode> {
    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    // This path is relative to the cargo root
    Ok(ServeDir::new(root)
        .precompressed_gzip()
        .precompressed_br()
        .oneshot(request)
        .await
        .into_response())
}
