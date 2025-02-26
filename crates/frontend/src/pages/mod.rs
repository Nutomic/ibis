use ibis_api_client::{CLIENT, article::GetArticleParams, errors::FrontendResult};
use ibis_database::common::{
    MAIN_PAGE_NAME,
    article::{ArticleView, EditView},
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

pub mod article;
pub mod instance;
pub mod user;

pub fn article_title_param() -> Option<String> {
    let params = use_params_map();
    params.get().get("title").clone()
}

fn article_resource() -> Resource<FrontendResult<ArticleView>> {
    Resource::new(article_title_param, move |title| async move {
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

async fn article_edits_resource(
    article: Resource<FrontendResult<ArticleView>>,
) -> Resource<FrontendResult<Vec<EditView>>> {
    let id = article.await.map(|a| a.article.id);
    Resource::new(
        move || article.get(),
        move |_| {
            let id = id.clone();
            async move { CLIENT.get_article_edits(id?).await }
        },
    )
}
