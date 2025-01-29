use crate::frontend::{
    components::{
        article_nav::{ActiveTab, ArticleNav},
        article_nav2::{ActiveTab2, ArticleNav2},
        suspense_error::SuspenseError,
    },
    markdown::render_article_markdown,
    pages::article_resource_result,
};
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::use_query_map;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let article = article_resource_result();
    let query = use_query_map();
    let edit_successful = query.get_untracked().get("edit_successful").is_some();

    view! {
        <ArticleNav2 article=article active_tab=ActiveTab2::Read />
        <SuspenseError>
            {move || Suspend::new(async move {
                let article = article.await;
                let markdown = article.map(|a| render_article_markdown(&a.article.text));
                if let Ok(markdown) = markdown {
                    Either::Right(
                        view! {
                            <div class="max-w-full prose prose-slate" inner_html=markdown></div>
                        },
                    )
                } else {
                    Either::Left(markdown)
                }
            })} <Show when=move || edit_successful>
                <div class="toast toast-center">
                    <div class="alert alert-success">Edit successful</div>
                </div>
            </Show>
        </SuspenseError>
    }
}
