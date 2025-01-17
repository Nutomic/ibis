use crate::{
    common::comment::DbComment,
    frontend::{
        components::{
            article_nav::{ActiveTab, ArticleNav},
            comment::CommentView,
            comment_editor::CommentEditorView,
        },
        pages::article_resource,
    },
};
use leptos::prelude::*;

#[component]
pub fn ArticleDiscussion() -> impl IntoView {
    let article = article_resource();

    // TODO: ArticleView should contain Person for each comment
    // TODO: allow creating nested reply
    view! {
        <ArticleNav article=article active_tab=ActiveTab::Discussion />
        <Suspense fallback=|| view! { "Loading..." }>
            <CommentEditorView article=article />
            <div>
                <For
                    each=move || article.get().map(|a| a.comments).unwrap_or_default()
                    key=|comment| comment.id
                    children=move |comment: DbComment| view! { <CommentView comment /> }
                />
            </div>
        </Suspense>
    }
}
