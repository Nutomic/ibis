use crate::frontend::markdown::render_markdown;
use leptos::{html::Textarea, prelude::*};

#[component]
pub fn EditorView(
    textarea_ref: NodeRef<Textarea>,
    content: Signal<String>,
    set_content: WriteSignal<String>,
) -> impl IntoView {
    let (preview, set_preview) = signal(render_markdown(&content.get_untracked()));
    let (show_preview, set_show_preview) = signal(false);

    view! {
        <div>
            <div class="my-4 w-full flex max-sm:flex-col">
                <textarea
                    prop:value=content
                    placeholder="Article text..."
                    class="grow textarea textarea-primary min-h-80 resize-none text-base text-base"
                    on:input=move |evt| {
                        let val = event_target_value(&evt);
                        set_preview.set(render_markdown(&val));
                        set_content.set(val);
                    }
                    node_ref=textarea_ref
                ></textarea>
                <Show when=move || { show_preview.get() }>
                    <div class="divider md:hidden"></div>
                    <div
                        class="prose prose-slate basis-6/12 md:ms-4 text-base py-2 max-sm:px-2"
                        inner_html=move || preview.get()
                    ></div>
                </Show>
            </div>
            <div class="flex h-min items-center mb-4">
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
