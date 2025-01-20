use crate::{
    common::comment::DbCommentView,
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

    // TODO: need to use correct order
    view! {
        <ArticleNav article=article active_tab=ActiveTab::Discussion />
        <Suspense fallback=|| view! { "Loading..." }>
            <CommentEditorView article=article parent_id=None />
            <div>
                <For
                    each=move || article.get().map(|a| a.comments).unwrap_or_default()
                    key=|comment| comment.comment.id
                    children=move |comment: DbCommentView| view! { <CommentView article comment /> }
                />
            </div>
        </Suspense>
    }
}
