use crate::frontend::{
    components::{
        article_nav::{ActiveTab, ArticleNav},
        edit_list::EditList,
        suspense_error::SuspenseError,
    },
    pages::{article_edits_resource, article_resource},
};
use leptos::prelude::*;

#[component]
pub fn ArticleHistory() -> impl IntoView {
    let article = article_resource();
    let edits = article_edits_resource(article);

    view! {
        <ArticleNav article=article active_tab=ActiveTab::History />
        <SuspenseError result=edits>
            {move || Suspend::new(async move {
                edits
                    .await
                    .map(|edits| {
                        view! { <EditList edits=edits for_article=true /> }
                    })
            })}

        </SuspenseError>
    }
}
