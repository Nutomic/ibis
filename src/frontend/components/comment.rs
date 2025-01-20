use crate::{
    common::{
        article::DbArticleView,
        comment::{DbCommentView, EditCommentForm},
    },
    frontend::{
        api::CLIENT,
        app::{site, DefaultResource},
        components::comment_editor::CommentEditorView,
        user_link,
    },
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

    let allow_delete = site().with_default(|site| site.my_profile.as_ref().map(|p| p.person.id))
        == Some(comment.comment.creator_id);
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
        <div style=style_>
            <div class="py-2">
                <div class="text-xs">{user_link(&comment.creator)}</div>
                <div class="my-2">{content}</div>
                <div class="text-xs">
                    <Show when=move || !comment.comment.deleted>
                        <a class="link" on:click=move |_| set_show_editor.update(|s| *s = !*s)>
                            Reply
                        </a>
                    </Show>
                    " | "
                    <Show when=move || allow_delete>
                        <a
                            class="link"
                            on:click=move |_| {
                                delete_restore_comment_action.dispatch(());
                            }
                        >
                            {delete_restore_label}
                        </a>
                    </Show>
                    <Show when=move || show_editor.get()>
                        <CommentEditorView
                            article=article
                            parent_id=Some(comment.comment.id)
                            set_show_editor=Some(set_show_editor)
                        />
                    </Show>
                </div>
            </div>
            <div class="m-0 divider"></div>
        </div>
    }
}
