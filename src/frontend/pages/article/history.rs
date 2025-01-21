use crate::frontend::{
    components::{
        article_nav::{ActiveTab, ArticleNav},
        edit_list::EditList,
    },
    pages::{article_edits_resource, article_resource},
};
use leptos::prelude::*;

#[component]
pub fn ArticleHistory() -> impl IntoView {
    let article = article_resource();

    view! {
        <ArticleNav article=article active_tab=ActiveTab::History />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || {
                article_edits_resource(article)
                    .get()
                    .map(|edits| {
                        view! { <EditList edits=edits for_article=true /> }
                    })
            }}

        </Suspense>
    }
}
