use crate::frontend::markdown::render_markdown;
use leptos::{ev::beforeunload, html::Textarea, prelude::*};
use leptos_use::{use_event_listener, use_window};

#[component]
pub fn EditorView(
    textarea_ref: NodeRef<Textarea>,
    content: Signal<String>,
    set_content: WriteSignal<String>,
) -> impl IntoView {
    let (preview, set_preview) = signal(render_markdown(&content.get_untracked()));
    let (show_preview, set_show_preview) = signal(false);

    // Prevent user from accidentally closing the page while editing. Doesnt prevent navigation
    // within Ibis.
    // https://github.com/Nutomic/ibis/issues/87
    let _ = use_event_listener(use_window(), beforeunload, |evt| {
        evt.stop_propagation();
        evt.prevent_default();
    });

    view! {
        <div>
            <div class="flex my-4 w-full max-sm:flex-col">
                <textarea
                    prop:value=content
                    placeholder="Article text..."
                    class="text-base text-base resize-none grow textarea textarea-primary min-h-80"
                    on:input=move |evt| {
                        let val = event_target_value(&evt);
                        set_preview.set(render_markdown(&val));
                        set_content.set(val);
                    }
                    node_ref=textarea_ref
                ></textarea>
                <Show when=move || { show_preview.get() }>
                    <div class="md:hidden divider"></div>
                    <div
                        class="py-2 text-base prose prose-slate basis-6/12 max-sm:px-2 md:ms-4"
                        inner_html=move || preview.get()
                    ></div>
                </Show>
            </div>
            <div class="flex items-center mb-4 h-min">
                <button
                    class="btn btn-secondary"
                    on:click=move |_| { set_show_preview.update(|s| *s = !*s) }
                >
                    Preview
                </button>
                <p class="mx-4">
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
