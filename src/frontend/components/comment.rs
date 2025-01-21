use crate::{
    common::{
        article::DbArticleView,
        comment::{DbCommentView, EditCommentForm},
        newtypes::CommentId,
    },
    frontend::{
        api::CLIENT,
        app::{site, DefaultResource},
        components::comment_editor::CommentEditorView,
        markdown::render_comment_markdown,
        time_ago, user_link,
    },
};
use leptos::prelude::*;

#[component]
pub fn CommentView(
    article: Resource<DbArticleView>,
    comment: DbCommentView,
    show_editor: (ReadSignal<CommentId>, WriteSignal<CommentId>),
) -> impl IntoView {
    // css class is not included because its dynamically generated, need to use raw css instead of class
    let margin = comment.comment.depth * 2;
    let style_ = format!("margin-left: {margin}rem;");

    let comment_id = format!("comment-{}", comment.comment.id.0);
    let comment_link = format!(
        "/article/{}/discussion#{comment_id}",
        article
            .get()
            .map(|a| a.article.title.clone())
            .unwrap_or_default(),
    );

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
        <div style=style_ id=comment_id>
            <div class="py-2">
                <div class="text-xs flex">
                    <span class="grow">{user_link(&comment.creator)}</span>
                    {time_ago(comment.comment.published)}
                </div>
                <div
                    class="my-2 prose prose-slate"
                    inner_html=render_comment_markdown(&content)
                ></div>
                <div class="text-xs">
                    <Show when=move || !comment.comment.deleted>
                        <a class="link" on:click=move |_| show_editor.1.set(comment.comment.id)>
                            Reply
                        </a>
                    </Show>
                    " | "
                    <a class="link" href=comment_link>
                        Link
                    </a>
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
                    <Show when=move || show_editor.0.get() == comment.comment.id>
                        <CommentEditorView
                            article=article
                            parent_id=Some(comment.comment.id)
                            set_show_editor=Some(show_editor.1)
                        />
                    </Show>
                </div>
            </div>
            <div class="m-0 divider"></div>
        </div>
    }
}
