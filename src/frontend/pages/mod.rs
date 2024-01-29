use crate::common::{ArticleView, GetArticleData};
use crate::frontend::app::GlobalState;
use leptos::{create_resource, Resource};

pub(crate) mod article;
pub(crate) mod diff;
pub mod login;
pub mod register;
pub(crate) mod search;

#[derive(Debug, Clone, Copy, Default)]
pub enum Page {
    #[default]
    Home,
    Login,
    Register,
}

impl Page {
    pub fn path(&self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Login => "/login",
            Self::Register => "/register",
        }
    }
}

fn article_resource(title: String) -> Resource<String, ArticleView> {
    create_resource(
        move || title.clone(),
        move |title| async move {
            GlobalState::api_client()
                .get_article(GetArticleData {
                    title: Some(title),
                    instance_id: None,
                    id: None,
                })
                .await
                .unwrap()
        },
    )
}
