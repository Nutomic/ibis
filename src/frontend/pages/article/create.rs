use crate::{
    common::CreateArticleForm,
    frontend::{api::CLIENT, components::editor::EditorView},
};
use html::Textarea;
use leptos::*;
use leptos_router::Redirect;
use leptos_use::{use_textarea_autosize, UseTextareaAutosizeReturn};

#[component]
pub fn CreateArticle() -> impl IntoView {
    let (title, set_title) = create_signal(String::new());
    let textarea_ref = create_node_ref::<Textarea>();
    let UseTextareaAutosizeReturn {
        content,
        set_content,
        trigger_resize: _,
    } = use_textarea_autosize(textarea_ref);
    let (summary, set_summary) = create_signal(String::new());
    let (create_response, set_create_response) = create_signal(None::<()>);
    let (create_error, set_create_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);
    let button_is_disabled =
        Signal::derive(move || wait_for_response.get() || summary.get().is_empty());
    let submit_action = create_action(move |(title, text, summary): &(String, String, String)| {
        let title = title.clone();
        let text = text.clone();
        let summary = summary.clone();
        async move {
            let form = CreateArticleForm {
                title,
                text,
                summary,
            };
            set_wait_for_response.update(|w| *w = true);
            let res = CLIENT.create_article(&form).await;
            set_wait_for_response.update(|w| *w = false);
            match res {
                Ok(_res) => {
                    set_create_response.update(|v| *v = Some(()));
                    set_create_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = err.0.to_string();
                    log::warn!("Unable to create: {msg}");
                    set_create_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    view! {
        <h1 class="text-4xl font-bold font-serif my-4">Create new Article</h1>
        <Show
            when=move || create_response.get().is_some()
            fallback=move || {
                view! {
                    <div class="item-view">
                        <input
                            class="input input-primary w-full"
                            type="text"
                            required
                            placeholder="Title"
                            prop:disabled=move || wait_for_response.get()
                            on:keyup=move |ev| {
                                let val = event_target_value(&ev);
                                set_title.update(|v| *v = val);
                            }
                        />

                        <EditorView textarea_ref content set_content />

                        {move || {
                            create_error
                                .get()
                                .map(|err| {
                                    view! { <p style="color:red;">{err}</p> }
                                })
                        }}

                        <div class="flex flex-row">
                            <input
                                class="input input-primary grow mr-4"
                                type="text"
                                placeholder="Edit summary"
                                on:keyup=move |ev| {
                                    let val = event_target_value(&ev);
                                    set_summary.update(|p| *p = val);
                                }
                            />

                            <button
                                class="btn btn-primary"
                                prop:disabled=move || button_is_disabled.get()
                                on:click=move |_| {
                                    submit_action
                                        .dispatch((title.get(), content.get(), summary.get()))
                                }
                            >
                                Submit
                            </button>
                        </div>
                    </div>
                }
            }
        >

            <Redirect path=format!("/article/{}", title.get().replace(' ', "_")) />
        </Show>
    }
}
