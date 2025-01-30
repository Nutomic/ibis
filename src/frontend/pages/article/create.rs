use crate::{
    common::article::CreateArticleParams,
    frontend::{
        api::CLIENT,
        components::article_editor::EditorView,
        utils::resources::{is_admin, site, DefaultResource},
    },
};
use leptos::{html::Textarea, prelude::*};
use leptos_meta::Title;
use leptos_router::{components::Redirect, hooks::use_query_map};
use leptos_use::{use_textarea_autosize, UseTextareaAutosizeReturn};

#[component]
pub fn CreateArticle() -> impl IntoView {
    let title = use_query_map()
        .get()
        .get("title")
        .unwrap_or_default()
        .replace('_', " ");
    let title = title.split_once('@').map(|(t, _)| t).unwrap_or(&title);
    let (title, set_title) = signal(title.to_string());

    let textarea_ref = NodeRef::<Textarea>::new();
    let UseTextareaAutosizeReturn {
        content,
        set_content,
        trigger_resize: _,
    } = use_textarea_autosize(textarea_ref);
    let (summary, set_summary) = signal(String::new());
    let (create_response, set_create_response) = signal(None::<()>);
    let (create_error, set_create_error) = signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = signal(false);
    let button_is_disabled =
        Signal::derive(move || wait_for_response.get() || summary.get().is_empty());
    let submit_action = Action::new(move |(title, text, summary): &(String, String, String)| {
        let title = title.clone();
        let text = text.clone();
        let summary = summary.clone();
        async move {
            let params = CreateArticleParams {
                title,
                text,
                summary,
            };
            set_wait_for_response.update(|w| *w = true);
            let res = CLIENT.create_article(&params).await;
            set_wait_for_response.update(|w| *w = false);
            match res {
                Ok(_res) => {
                    set_create_response.update(|v| *v = Some(()));
                    set_create_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = err.to_string();
                    log::warn!("Unable to create: {msg}");
                    set_create_error.update(|e| *e = Some(msg));
                }
            }
        }
    });
    let show_approval_message = Signal::derive(move || {
        site().with_default(|site| site.config.article_approval) && !is_admin()
    });

    view! {
        <Title text="Create new Article" />
        <h1 class="my-4 font-serif text-4xl font-bold">Create new Article</h1>
        <Suspense>
            <Show when=move || show_approval_message.get()>
                <div class="mb-4 alert alert-warning">
                    New articles require admin approval before being published
                </div>
            </Show>
        </Suspense>
        <Show
            when=move || create_response.get().is_some()
            fallback=move || {
                view! {
                    <div class="item-view">
                        <input
                            class="w-full input input-primary"
                            type="text"
                            required
                            placeholder="Title"
                            value=title
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
                                class="mr-4 input input-primary grow"
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
                                        .dispatch((title.get(), content.get(), summary.get()));
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
