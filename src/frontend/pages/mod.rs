use super::utils::errors::FrontendResult;
use crate::{
    common::{
        article::{DbArticleView, EditView, GetArticleParams},
        MAIN_PAGE_NAME,
    },
    frontend::api::CLIENT,
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

pub mod article;
pub mod instance;
pub mod user;

fn article_resource() -> Resource<DbArticleView> {
    let params = use_params_map();
    let title = move || params.get().get("title").clone();
    Resource::new(title, move |title| async move {
        let mut title = title.unwrap_or(MAIN_PAGE_NAME.to_string());
        let mut domain = None;
        if let Some((title_, domain_)) = title.clone().split_once('@') {
            title = title_.to_string();
            domain = Some(domain_.to_string());
        }
        CLIENT
            .get_article(GetArticleParams {
                title: Some(title),
                domain,
                id: None,
            })
            .await
            .unwrap()
    })
}
fn article_resource_result() -> Resource<FrontendResult<DbArticleView>> {
    let params = use_params_map();
    let title = move || params.get().get("title").clone();
    Resource::new(title, move |title| async move {
        let mut title = title.unwrap_or(MAIN_PAGE_NAME.to_string());
        let mut domain = None;
        if let Some((title_, domain_)) = title.clone().split_once('@') {
            title = title_.to_string();
            domain = Some(domain_.to_string());
        }
        CLIENT
            .get_article(GetArticleParams {
                title: Some(title),
                domain,
                id: None,
            })
            .await
    })
}

fn article_edits_resource(article: Resource<DbArticleView>) -> Resource<Vec<EditView>> {
    Resource::new(
        move || article.get(),
        move |_| async move {
            CLIENT
                .get_article_edits(article.await.article.id)
                .await
                .unwrap_or_default()
        },
    )
}
