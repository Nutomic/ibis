use crate::{
    common::{
        article::ArticleView,
        comment::{Comment, CommentView, EditCommentParams},
        newtypes::CommentId,
    },
    frontend::{
        api::CLIENT,
        components::comment_editor::{CommentEditorView, EditParams},
        markdown::render_comment_markdown,
        utils::{
            errors::{FrontendResult, FrontendResultExt},
            formatting::{comment_path, time_ago, user_link},
            resources::my_profile,
        },
    },
};
use leptos::prelude::*;
use phosphor_leptos::{Icon, ARROW_BEND_UP_LEFT, FEDIVERSE_LOGO, LINK, PENCIL, TRASH};

#[component]
pub fn CommentView(
    article: Resource<FrontendResult<ArticleView>>,
    comment: CommentView,
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
            .and_then(|a| a.ok())
            .map(|a| comment_path(&comment.comment, &a.article))
            .unwrap_or_default(),
    );

    let delete_restore_comment_action = Action::new(move |_: &()| async move {
        let params = EditCommentParams {
            id: comment.comment.id,
            deleted: Some(!comment_change_signal.0.get_untracked().deleted),
            content: None,
        };
        CLIENT
            .edit_comment(&params)
            .await
            .error_popup(|comment| comment_change_signal.1.set(comment.comment));
    });

    let is_creator =
        my_profile().map(|my_profile| my_profile.person.id) == Some(comment.comment.creator_id);

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
                        <Icon icon=LINK />
                        <span class="ml-2">{time_ago(comment.comment.published)}</span>
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
                    <div class="mt-2 max-w-full prose prose-slate" inner_html=render_comment></div>
                    <div class="grid grid-cols-5 grid-rows-1 gap-2 w-fit text-s">
                        <Show when=move || !comment.comment.deleted>
                            <a
                                class="link"
                                on:click=move |_| show_editor.1.set(comment.comment.id)
                                title="Reply"
                            >
                                <Icon icon=ARROW_BEND_UP_LEFT />
                            </a>
                        </Show>
                        <a class="link" href=comment.comment.ap_id.to_string() title="Fedilink">
                            <Icon icon=FEDIVERSE_LOGO />
                        </a>
                        <Show when=move || is_creator && !comment_change_signal.0.get().deleted>
                            <a
                                class="link"
                                title="Edit"
                                on:click=move |_| {
                                    is_editing.1.set(true);
                                }
                            >
                                <Icon icon=PENCIL />
                            </a>
                        </Show>
                        <Show when=move || is_creator>
                            <a
                                class="link"
                                on:click=move |_| {
                                    delete_restore_comment_action.dispatch(());
                                }
                                title=delete_restore_label
                            >
                                <Icon icon=TRASH />
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

fn render_content(comment: Comment) -> String {
    let content = if comment.deleted {
        "*deleted*"
    } else {
        &comment.content
    };
    render_comment_markdown(content)
}

fn delete_restore_label(comment: Comment) -> &'static str {
    if comment.deleted {
        "Restore"
    } else {
        "Delete"
    }
}
