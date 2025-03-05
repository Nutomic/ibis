use crate::{components::prevent_navigation, utils::use_cookie};
use ibis_markdown::render_article_markdown;
use leptos::{html::Textarea, prelude::*};

#[component]
pub fn EditorView(
    textarea_ref: NodeRef<Textarea>,
    content: Signal<String>,
    set_content: WriteSignal<String>,
) -> impl IntoView {
    let (preview, set_preview) = signal(render_article_markdown(&content.get_untracked()));
    let cookie = use_cookie("editor_preview");
    let show_preview = Signal::derive(move || cookie.0.get().unwrap_or(true));

    prevent_navigation(content);

    view! {
        <div>
            <div class="flex my-4 w-full max-sm:flex-col">
                <textarea
                    prop:value=content
                    placeholder="Article text..."
                    class="text-base resize-none grow textarea textarea-primary min-h-80"
                    on:input=move |evt| {
                        let val = event_target_value(&evt);
                        set_preview.set(render_article_markdown(&val));
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
                    class="btn btn-secondary btn-sm"
                    on:click=move |_| {
                        cookie.1.set(Some(!show_preview.get_untracked()));
                    }
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
