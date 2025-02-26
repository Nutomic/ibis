use ibis_api_client::{
    CLIENT,
    comment::{CreateCommentParams, EditCommentParams},
    errors::{FrontendResult, FrontendResultExt},
};
use ibis_database::common::{article::ArticleView, comment::Comment, newtypes::CommentId};
use leptos::{html::Textarea, prelude::*};
use leptos_use::{UseTextareaAutosizeReturn, use_textarea_autosize};

#[derive(Clone)]
pub struct EditParams {
    pub comment: Comment,
    pub set_comment: WriteSignal<Comment>,
    pub set_is_editing: WriteSignal<bool>,
}

#[component]
pub fn CommentEditorView(
    article: Resource<FrontendResult<ArticleView>>,
    #[prop(optional)] parent_id: Option<CommentId>,
    /// Set this to CommentId(-1) to hide all editors
    #[prop(optional)]
    set_show_editor: Option<WriteSignal<CommentId>>,
    /// If this is present we are editing an existing comment
    #[prop(optional)]
    edit_params: Option<EditParams>,
) -> impl IntoView {
    let textarea_ref = NodeRef::<Textarea>::new();
    let UseTextareaAutosizeReturn {
        content,
        set_content,
        trigger_resize: _,
    } = use_textarea_autosize(textarea_ref);
    let set_is_editing = edit_params.as_ref().map(|e| e.set_is_editing);
    if let Some(edit_params) = &edit_params {
        set_content.set(edit_params.comment.content.clone())
    };

    let submit_comment_action = Action::new(move |_: &()| {
        let edit_params = edit_params.clone();
        async move {
            if let Some(edit_params) = edit_params {
                let params = EditCommentParams {
                    id: edit_params.comment.id,
                    content: Some(content.get_untracked()),
                    deleted: None,
                };
                CLIENT.edit_comment(&params).await.error_popup(|comment| {
                    edit_params.set_comment.set(comment.comment);
                    edit_params.set_is_editing.set(false);
                });
            } else {
                let params = CreateCommentParams {
                    content: content.get_untracked(),
                    article_id: article.await.map(|a| a.article.id).unwrap_or_default(),
                    parent_id,
                };
                CLIENT.create_comment(&params).await.error_popup(|_| {
                    article.refetch();
                    if let Some(set_show_editor) = set_show_editor {
                        set_show_editor.set(CommentId(-1));
                    }
                });
            }
        }
    });

    view! {
        <div class="my-2">
            <textarea
                prop:value=content
                placeholder="Your comment..."
                class="w-full resize-none textarea textarea-secondary min-h-10"
                on:input=move |evt| {
                    let val = event_target_value(&evt);
                    set_content.set(val);
                }
                node_ref=textarea_ref
            ></textarea>
            <div class="flex items-center mt-2 h-min">
                <button
                    class="btn btn-secondary btn-sm"
                    on:click=move |_| {
                        submit_comment_action.dispatch(());
                    }
                >
                    Submit
                </button>
                <Show when=move || set_show_editor.is_some()>
                    <button
                        class="ml-2 btn btn-secondary btn-sm"
                        on:click=move |_| {
                            if let Some(set_show_editor) = set_show_editor {
                                set_show_editor.set(CommentId(-1));
                            }
                            if let Some(set_is_editing) = set_is_editing {
                                set_is_editing.set(false);
                            }
                        }
                    >
                        Cancel
                    </button>
                </Show>
                <p class="mx-2">
                    <a
                        class="link link-secondary"
                        href="https://ibis.wiki/article/Markdown_Guide"
                        target="blank_"
                    >
                        Markdown
                    </a>
                    " formatting is supported"
                </p>
            </div>
        </div>
    }
}
