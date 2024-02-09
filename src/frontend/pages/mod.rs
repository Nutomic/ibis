use crate::common::{ArticleView, GetArticleData, MAIN_PAGE_NAME};
use crate::frontend::app::GlobalState;
use leptos::{create_resource, Resource};

pub(crate) mod article;
pub(crate) mod diff;
pub(crate) mod instance_details;
pub(crate) mod login;
pub(crate) mod register;
pub(crate) mod search;

fn article_resource(
    title: impl Fn() -> Option<String> + 'static,
) -> Resource<Option<String>, ArticleView> {
    create_resource(title, move |title| async move {
        let mut title = title.unwrap_or(MAIN_PAGE_NAME.to_string());
        let mut domain = None;
        if let Some((title_, domain_)) = title.clone().split_once('@') {
            title = title_.to_string();
            domain = Some(domain_.to_string());
        }
        GlobalState::api_client()
            .get_article(GetArticleData {
                title: Some(title),
                instance_domain: domain,
                id: None,
            })
            .await
            .unwrap()
    })
}
