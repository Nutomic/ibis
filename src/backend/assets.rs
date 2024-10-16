use super::error::MyResult;
use anyhow::anyhow;
use axum::{
    body::Body,
    extract::Path,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use axum_macros::debug_handler;
use include_dir::include_dir;
use once_cell::sync::OnceCell;
use reqwest::header::HeaderMap;
use std::fs::read_to_string;

pub fn asset_routes() -> MyResult<Router<()>> {
    Ok(Router::new()
        .route("/assets/ibis.css", get(ibis_css))
        .route(
            "/assets/simple.css",
            get((css_headers(), include_str!("../../assets/simple.css"))),
        )
        .route(
            "/assets/katex.min.css",
            get((css_headers(), include_str!("../../assets/katex.min.css"))),
        )
        .route("/assets/fonts/*font", get(get_font))
        .route(
            "/assets/index.html",
            get(include_str!("../../assets/index.html")),
        )
        .route("/pkg/ibis.js", get(serve_js))
        .route("/pkg/ibis_bg.wasm", get(serve_wasm)))
}

fn css_headers() -> HeaderMap {
    static INSTANCE: OnceCell<HeaderMap> = OnceCell::new();
    INSTANCE
        .get_or_init(|| {
            let mut css_headers = HeaderMap::new();
            let val = "text/css".parse().expect("valid header value");
            css_headers.insert("Content-Type", val);
            css_headers
        })
        .clone()
}

async fn ibis_css() -> MyResult<(HeaderMap, Response<Body>)> {
    let res = if cfg!(debug_assertions) {
        read_to_string("assets/ibis.css")?.into_response()
    } else {
        include_str!("../../assets/ibis.css").into_response()
    };
    Ok((css_headers(), res))
}

#[debug_handler]
async fn serve_js() -> MyResult<impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/javascript".parse()?);
    let content = include_str!("../../assets/dist/ibis.js");
    Ok((headers, content))
}

#[debug_handler]
async fn serve_wasm() -> MyResult<impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/wasm".parse()?);
    let content = include_bytes!("../../assets/dist/ibis_bg.wasm");
    Ok((headers, content))
}

#[debug_handler]
async fn get_font(Path(font): Path<String>) -> MyResult<impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert("Content-type", "font/woff2".parse()?);
    let font_dir = include_dir!("assets/fonts");
    if let Some(font_file) = font_dir.get_file(font) {
        return Ok((headers, font_file.contents()));
    }
    Err(anyhow!("invalid font").into())
}
