use crate::common::utils::extract_domain;
use crate::common::{DbArticle, DbPerson};
use leptos::*;

pub mod api;
pub mod app;
mod components;
pub mod error;
pub mod markdown;
pub mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {}

fn article_link(article: &DbArticle) -> String {
    if article.local {
        format!("/article/{}", article.title)
    } else {
        format!(
            "/article/{}@{}",
            article.title,
            extract_domain(&article.ap_id)
        )
    }
}

fn article_title(article: &DbArticle) -> String {
    let title = article.title.replace('_', " ");
    if article.local {
        title
    } else {
        format!("{}@{}", title, extract_domain(&article.ap_id))
    }
}

fn user_title(person: &DbPerson) -> String {
    if person.local {
        person.username.clone()
    } else {
        format!("{}@{}", person.username, extract_domain(&person.ap_id))
    }
}

fn user_link(person: &DbPerson) -> impl IntoView {
    let creator_path = format!("/user/{}", person.username);
    view! {
        <a href={creator_path}>{user_title(person)}</a>
    }
}

fn backend_hostname() -> String {
    let backend_hostname;
    #[cfg(not(feature = "ssr"))]
    {
        backend_hostname = web_sys::window().unwrap().location().host().unwrap();
    }
    #[cfg(feature = "ssr")]
    {
        backend_hostname = crate::backend::config::IbisConfig::read().bind.to_string();
    }
    backend_hostname
}
