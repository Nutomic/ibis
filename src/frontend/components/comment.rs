use crate::common::comment::DbComment;
use leptos::prelude::*;

#[component]
pub fn CommentView(comment: DbComment) -> impl IntoView {
    // css class is not included because its dynamically generated, need to use raw css instead of class
    let margin = comment.depth * 4;
    let class_ = format!("pl-{margin}");
    view! { <div class="pl-4">{comment.content}</div> }
    // TODO: actions for reply, delete, undelete
}
