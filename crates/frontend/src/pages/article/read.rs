use crate::pages::article_resource;
use ibis_frontend_components::{
    article_nav::{ActiveTab, ArticleNav},
    suspense_error::SuspenseError,
};
use ibis_markdown::render_article_markdown;
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::use_query_map;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let article = article_resource();
    let query = use_query_map();
    let edit_successful = query.get_untracked().get("edit_successful").is_some();

    view! {
        <ArticleNav article=article active_tab=ActiveTab::Read />
        <SuspenseError result=article>
            {move || Suspend::new(async move {
                let article = article.await;
                let markdown = article.map(|a| render_article_markdown(&a.article.text));
                if let Ok(markdown) = markdown {
                    Either::Right(
                        view! {
                            <div
                                class="max-w-full prose prose-slate text-ellipsis overflow-x-hidden"
                                inner_html=markdown
                            ></div>
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
