use crate::{
    common::{article::DbArticleView, comment::CreateCommentForm, newtypes::CommentId},
    frontend::api::CLIENT,
};
use leptos::{html::Textarea, prelude::*};
use leptos_use::{use_textarea_autosize, UseTextareaAutosizeReturn};

#[component]
pub fn CommentEditorView(
    article: Resource<DbArticleView>,
    parent_id: Option<CommentId>,
    set_show_editor: Option<WriteSignal<bool>>,
) -> impl IntoView {
    let textarea_ref = NodeRef::<Textarea>::new();
    let UseTextareaAutosizeReturn {
        content,
        set_content,
        trigger_resize: _,
    } = use_textarea_autosize(textarea_ref);

    let submit_comment_action = Action::new(move |_: &()| async move {
        let form = CreateCommentForm {
            content: content.get_untracked(),
            article_id: article.await.article.id,
            parent_id,
        };
        CLIENT.create_comment(&form).await.unwrap();
        if let Some(set_show_editor) = set_show_editor {
            set_show_editor.set(false);
        }
        article.refetch();
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
                                set_show_editor.set(false);
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
