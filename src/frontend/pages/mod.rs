use crate::common::{ArticleView, GetArticleData};
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
        let title = title.unwrap_or("Main_Page".to_string());
        GlobalState::api_client()
            .get_article(GetArticleData {
                title: Some(title),
                instance_id: None,
                id: None,
            })
            .await
            .unwrap()
    })
}
