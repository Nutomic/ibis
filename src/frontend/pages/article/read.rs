use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    markdown::render_markdown,
    pages::{article::table_of_contents::Toc, article_resource},
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let article = article_resource();
    let query = use_query_map();
    let edit_successful = query.get_untracked().get("edit_successful").is_some();

    view! {
        <ArticleNav article=article active_tab=ActiveTab::Read />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>

            {move || {
                article
                    .get()
                    .map(|article| {
                        view! {
                            <div>
                                <Toc text=article.article.text.clone() />
                                <div
                                    class="max-w-full prose prose-slate"
                                    inner_html=render_markdown(&article.article.text)
                                ></div>

                            </div>
                        }
                    })
            }} <Show when=move || edit_successful>
                <div class="toast toast-center">
                    <div class="alert alert-success">Edit successful</div>
                </div>
            </Show>
        </Suspense>
    }
}
