use crate::frontend::components::article_nav::ArticleNav;
use crate::frontend::pages::article_resource;
use crate::frontend::{article_title, user_link};
use leptos::*;
use leptos_router::*;

#[component]
pub fn ArticleHistory() -> impl IntoView {
    let params = use_params_map();
    let title = move || params.get().get("title").cloned();
    let article = article_resource(title);

    view! {
        <ArticleNav article=article/>
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || article.get().map(|article| {
                view! {
                    <div class="item-view">
                        <h1>{article_title(&article.article)}</h1>
                        {
                            article.edits.into_iter().rev().map(|edit| {
                                let path = format!("/article/{}/diff/{}", article.article.title, edit.edit.hash.0);
                                let label = format!("{} ({})", edit.edit.summary, edit.edit.created.to_rfc2822());
                                view! {<li><a href={path}>{label}</a>" by "{user_link(&edit.creator)}</li> }
                            }).collect::<Vec<_>>()
                        }
                    </div>
                }
            })
        }
        </Suspense>
    }
}
