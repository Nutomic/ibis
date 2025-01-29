use crate::frontend::{
    components::article_nav::{ActiveTab, ArticleNav},
    markdown::render_article_markdown,
    pages::article_resource_result,
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let article = article_resource_result();
    let query = use_query_map();
    let edit_successful = query.get_untracked().get("edit_successful").is_some();

    //<ArticleNav article=article active_tab=ActiveTab::Read />
    view! {
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            <ErrorBoundary fallback=|errors| {
                view! {
                    <div class="error">
                        {move || {
                            errors
                                .get()
                                .into_iter()
                                .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                                .collect::<Vec<_>>()
                        }}
                    </div>
                }
            }>

                {move || Suspend::new(async move {
                    let article = article.await;
                    let markdown = article.map(|a| render_article_markdown(&a.article.text));
                    view! { {markdown} }
                })} <Show when=move || edit_successful>
                    <div class="toast toast-center">
                        <div class="alert alert-success">Edit successful</div>
                    </div>
                </Show>
            </ErrorBoundary>
        </Suspense>
    }
}
