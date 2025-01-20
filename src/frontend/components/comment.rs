use crate::{
    common::{
        article::DbArticleView,
        comment::{DbCommentView, EditCommentForm},
    },
    frontend::{api::CLIENT, components::comment_editor::CommentEditorView, user_link},
};
use leptos::prelude::*;

#[component]
pub fn CommentView(article: Resource<DbArticleView>, comment: DbCommentView) -> impl IntoView {
    // css class is not included because its dynamically generated, need to use raw css instead of class
    let margin = comment.comment.depth * 2;
    let style_ = format!("margin-left: {margin}rem;");

    let (show_editor, set_show_editor) = signal(false);

    let delete_restore_comment_action = Action::new(move |_: &()| async move {
        let form = EditCommentForm {
            id: comment.comment.id,
            deleted: Some(!comment.comment.deleted),
            content: None,
        };
        CLIENT.edit_comment(&form).await.unwrap();
        // TODO: Somehow the refetch doesnt change content to deleted as expected, and anyway
        //       its inefficient.
        article.refetch();
    });

    let content = if comment.comment.deleted {
        "**deleted**".to_string()
    } else {
        comment.comment.content.clone()
    };
    let delete_restore_label = if comment.comment.deleted {
        "Restore"
    } else {
        "Delete"
    };

    view! {
        <div class="py-2 pl-4" style=style_>
            <div class="text-sm">{user_link(&comment.creator)}</div>
            <div>{content}</div>
            <div class="text-sm">
                <a class="link" on:click=move |_| set_show_editor.update(|s| *s = !*s)>
                    Reply
                </a>
                " | "
                <a
                    class="link"
                    on:click=move |_| {
                        delete_restore_comment_action.dispatch(());
                    }
                >
                    {delete_restore_label}
                </a>
                <Show when=move || show_editor.get()>
                    <CommentEditorView article=article parent_id=Some(comment.comment.id) />
                </Show>
            </div>
            <div class="m-0 divider"></div>
        </div>
    }
    // TODO: actions for reply, delete, undelete
}
