use crate::frontend::markdown::render_markdown;
use html::Textarea;
use leptos::*;

#[component]
pub fn EditorView(
    // this param gives a false warning about being unused, ignore that
    #[allow(unused)] textarea_ref: NodeRef<Textarea>,
    content: Signal<String>,
    set_content: WriteSignal<String>,
) -> impl IntoView {
    let (preview, set_preview) = create_signal(render_markdown(&content.get()));
    let (show_preview, set_show_preview) = create_signal(false);

    view! {
        <textarea
            value=content
            placeholder="Article text..."
            class="textarea textarea-bordered textarea-primary min-w-full"
            on:input=move |evt| {
                let val = event_target_value(&evt);
                set_preview.set(render_markdown(&val));
                set_content.set(val);
            }
            node_ref=textarea_ref
        >
            {content.get()}
        </textarea>
        <button class="btn" on:click=move |_| { set_show_preview.update(|s| *s = !*s) }>
            Preview
        </button>
        <Show when=move || { show_preview.get() }>
            <div id="preview" inner_html=move || preview.get()></div>
        </Show>
        <div>
            <a href="https://commonmark.org/help/" target="blank_">
                Markdown
            </a>
            " formatting is supported"
        </div>
    }
}
