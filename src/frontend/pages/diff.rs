use crate::frontend::components::article_nav::ArticleNav;
use crate::frontend::pages::article_resource;
use leptos::*;
use leptos_router::*;

#[component]
pub fn EditDiff() -> impl IntoView {
    let params = use_params_map();
    let title = params.get_untracked().get("title").cloned().unwrap();
    let article = article_resource(title);

    view! {
        <ArticleNav article=article.clone()/>
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || article.get().map(|article| {
                let hash = params
                    .get_untracked()
                    .get("hash")
                    .cloned().unwrap();
                let edit = article.edits.iter().find(|e| e.hash.0.to_string() == hash).unwrap();
                // TODO: need to show username
                view! {
                    <div class="item-view">
                        <h1>{article.article.title.replace('_', " ")}</h1>
                        <pre>{edit.diff.clone()}</pre>
                    </div>
                }
            })
        }
        </Suspense>
    }
}
