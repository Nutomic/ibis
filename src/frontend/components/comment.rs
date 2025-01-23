use crate::{
    common::{
        article::DbArticleView,
        comment::{DbComment, DbCommentView, EditCommentParams},
        newtypes::CommentId,
    },
    frontend::{
        api::CLIENT,
        app::{site, DefaultResource},
        components::comment_editor::{CommentEditorView, EditParams},
        markdown::render_comment_markdown,
        time_ago,
        user_link,
    },
};
use leptos::prelude::*;

#[component]
pub fn CommentView(
    article: Resource<DbArticleView>,
    comment: DbCommentView,
    show_editor: (ReadSignal<CommentId>, WriteSignal<CommentId>),
) -> impl IntoView {
    let is_editing = signal(false);
    let comment_change_signal = signal(comment.comment.clone());
    let render_comment = move || render_content(comment_change_signal.0.get());
    let delete_restore_label = move || delete_restore_label(comment_change_signal.0.get());

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
        let params = EditCommentParams {
            id: comment.comment.id,
            deleted: Some(!comment_change_signal.0.get_untracked().deleted),
            content: None,
        };
        let comment = CLIENT.edit_comment(&params).await.unwrap();
        comment_change_signal.1.set(comment.comment);
    });

    let is_creator = site().with_default(|site| site.my_profile.as_ref().map(|p| p.person.id))
        == Some(comment.comment.creator_id);

    let edit_params = EditParams {
        comment: comment.comment.clone(),
        set_comment: comment_change_signal.1,
        set_is_editing: is_editing.1,
    };
    view! {
        <div style=style_ id=comment_id>
            <div class="py-2">
                <div class="flex text-xs">
                    <span class="grow">{user_link(&comment.creator)}</span>
                    <a href=comment_link class="link">
                        {time_ago(comment.comment.published)}
                    </a>
                </div>
                <Show
                    when=move || !is_editing.0.get()
                    fallback=move || {
                        view! {
                            <CommentEditorView
                                article=article
                                parent_id=comment.comment.id
                                set_show_editor=show_editor.1
                                edit_params=edit_params.clone()
                            />
                        }
                    }
                >
                    <div class="my-2 prose prose-slate" inner_html=render_comment></div>
                    <div class="text-xs">
                        <Show when=move || !comment.comment.deleted>
                            <a class="link" on:click=move |_| show_editor.1.set(comment.comment.id)>
                                Reply
                            </a>
                            " | "
                        </Show>
                        <a class="link" href=comment.comment.ap_id.to_string()>
                            Fedilink
                        </a>
                        " | "
                        <Show when=move || is_creator && !comment_change_signal.0.get().deleted>
                            <a
                                class="link"
                                on:click=move |_| {
                                    is_editing.1.set(true);
                                }
                            >
                                Edit
                            </a>
                            " | "
                        </Show>
                        <Show when=move || is_creator>
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
                                parent_id=comment.comment.id
                                set_show_editor=show_editor.1
                            />
                        </Show>
                    </div>
                </Show>
            </div>
            <div class="m-0 divider"></div>
        </div>
    }
}

fn render_content(comment: DbComment) -> String {
    let content = if comment.deleted {
        "*deleted*"
    } else {
        &comment.content
    };
    render_comment_markdown(content)
}

fn delete_restore_label(comment: DbComment) -> &'static str {
    if comment.deleted {
        "Restore"
    } else {
        "Delete"
    }
}
