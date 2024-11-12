use crate::{
    common::{ArticleView, GetArticleForm, MAIN_PAGE_NAME},
    frontend::app::GlobalState,
};
use leptos::{create_resource, Resource, SignalGet};
use leptos_router::use_params_map;

pub(crate) mod article;
pub(crate) mod diff;
pub(crate) mod instance;
pub(crate) mod login;
pub(crate) mod notifications;
pub(crate) mod register;
pub(crate) mod search;
pub(crate) mod user_profile;

fn article_resource() -> Resource<Option<String>, ArticleView> {
    let params = use_params_map();
    let title = move || params.get().get("title").cloned();
    create_resource(title, move |title| async move {
        let mut title = title.unwrap_or(MAIN_PAGE_NAME.to_string());
        let mut domain = None;
        if let Some((title_, domain_)) = title.clone().split_once('@') {
            title = title_.to_string();
            domain = Some(domain_.to_string());
        }
        GlobalState::api_client()
            .get_article(GetArticleForm {
                title: Some(title),
                domain,
                id: None,
            })
            .await
            .unwrap()
    })
}
