use crate::frontend::{
    components::{
        article_nav2::{ActiveTab2, ArticleNav2},
        edit_list::EditList,
        suspense_error::SuspenseError,
    },
    pages::{article_edits_resource, article_resource_result},
};
use leptos::prelude::*;

#[component]
pub fn ArticleHistory() -> impl IntoView {
    let article = article_resource_result();
    let edits = article_edits_resource(article);

    view! {
        <ArticleNav2 article=article active_tab=ActiveTab2::History />
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
