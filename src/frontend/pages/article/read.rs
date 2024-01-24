use crate::frontend::components::article_nav::ArticleNav;
use crate::frontend::pages::article_resource;
use leptos::*;
use leptos_router::*;

#[component]
pub fn ReadArticle() -> impl IntoView {
    let params = use_params_map();
    let title = params
        .get_untracked()
        .get("title")
        .cloned()
        .unwrap_or("Main_Page".to_string());
    let article = article_resource(title);

    view! {
        <ArticleNav article=article.clone()/>
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || article.get().map(|article|
            view! {
                <div class="item-view">
                    <h1>{article.article.title.replace('_', " ")}</h1>
                    <div>{article.article.text}</div>
                </div>
            })
        }
        </Suspense>
    }
}
