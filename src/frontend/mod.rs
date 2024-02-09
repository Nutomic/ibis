use crate::common::DbArticle;
use url::Url;

pub mod api;
pub mod app;
mod components;
pub mod error;
pub mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {}

fn extract_hostname(article: &DbArticle) -> String {
    let ap_id: Url;
    #[cfg(not(feature = "ssr"))]
    {
        ap_id = article.ap_id.parse().unwrap();
    }
    #[cfg(feature = "ssr")]
    {
        ap_id = article.ap_id.inner().clone();
    }
    let mut port = String::new();
    if let Some(port_) = ap_id.port() {
        port = format!(":{port_}");
    }
    format!("{}{port}", ap_id.host_str().unwrap())
}

fn article_link(article: &DbArticle) -> String {
    if article.local {
        format!("/article/{}", article.title)
    } else {
        format!("/article/{}@{}", article.title, extract_hostname(article))
    }
}

fn article_title(article: &DbArticle) -> String {
    let title = article.title.replace('_', " ");
    if article.local {
        title
    } else {
        format!("{}@{}", title, extract_hostname(article))
    }
}
