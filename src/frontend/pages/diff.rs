use crate::frontend::components::article_nav::ArticleNav;
use crate::frontend::pages::article_resource;
use crate::frontend::user_link;
use leptos::*;
use leptos_router::*;

#[component]
pub fn EditDiff() -> impl IntoView {
    let params = use_params_map();
    let title = move || params.get().get("title").cloned();
    let article = article_resource(title);

    view! {
        <ArticleNav article=article/>
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || article.get().map(|article| {
                let hash = params
                    .get_untracked()
                    .get("hash")
                    .cloned().unwrap();
                let edit = article.edits.iter().find(|e| e.edit.hash.0.to_string() == hash).unwrap();
                let label = format!("{} ({})", edit.edit.summary, edit.edit.created.to_rfc2822());

                view! {
                    <div class="item-view">
                        <h1>{article.article.title.replace('_', " ")}</h1>
                        <h2>{label}</h2>
                        <p>"by "{user_link(&edit.creator)}</p>
                        <pre>{edit.edit.diff.clone()}</pre>
                    </div>
                }
            })
        }
        </Suspense>
    }
}
