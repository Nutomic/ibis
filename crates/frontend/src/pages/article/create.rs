use ibis_api_client::{
    CLIENT,
    article::{CreateArticleParams, ImportArticleParams},
};
use ibis_database::common::{article::ArticleView, newtypes::InstanceId};
use ibis_frontend_components::{
    article_editor::EditorView,
    suspense_error::SuspenseError,
    utils::{formatting::article_path, i18n::IbisTitle},
};
use itertools::Itertools;
use leptos::{html::Textarea, prelude::*};
use leptos_fluent::tr;
use leptos_router::{components::Redirect, hooks::use_query_map};
use leptos_use::{UseTextareaAutosizeReturn, use_textarea_autosize};

#[component]
pub fn CreateArticle() -> impl IntoView {
    let title = use_query_map()
        .get_untracked()
        .get("title")
        .unwrap_or_default()
        .replace('_', " ");
    let title = title.split_once('@').map(|(t, _)| t).unwrap_or(&title);
    let title = signal(title.to_string());

    let textarea_ref = NodeRef::<Textarea>::new();
    let UseTextareaAutosizeReturn {
        content,
        set_content,
        trigger_resize: _,
    } = use_textarea_autosize(textarea_ref);
    let summary = signal(String::new());
    let import_url = signal(String::new());
    let instance_id = signal(String::new());
    let (create_response, set_create_response) = signal(None::<ArticleView>);
    let (create_error, set_create_error) = signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = signal(false);
    let create_button_is_disabled = Signal::derive(move || {
        wait_for_response.get() || summary.0.get().is_empty() || title.0.get().is_empty()
    });
    let import_button_is_disabled =
        Signal::derive(move || wait_for_response.get() || import_url.0.get().is_empty());
    let create_action = Action::new(
        move |(title, text, summary, instance_id): &(String, String, String, String)| {
            let params = CreateArticleParams {
                title: title.clone(),
                text: text.clone(),
                summary: summary.clone(),
                instance_id: Some(InstanceId(instance_id.clone().parse().unwrap_or(1))),
            };
            async move {
                set_wait_for_response.update(|w| *w = true);
                let res = CLIENT.create_article(&params).await;
                set_wait_for_response.update(|w| *w = false);
                match res {
                    Ok(res) => {
                        set_create_response.update(|v| *v = Some(res));
                        set_create_error.update(|e| *e = None);
                    }
                    Err(err) => {
                        let msg = err.to_string();
                        log::warn!("Unable to create: {msg}");
                        set_create_error.update(|e| *e = Some(msg));
                    }
                }
            }
        },
    );
    let import_action = Action::new(move |url: &String| {
        let params = ImportArticleParams {
            url: url.to_string(),
        };
        async move {
            set_wait_for_response.update(|w| *w = true);
            let res = CLIENT.import_article(&params).await;
            set_wait_for_response.update(|w| *w = false);
            match res {
                Ok(res) => {
                    set_create_response.update(|v| *v = Some(res));
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
    let instances = Resource::new(move || (), |_| async move { CLIENT.list_instances().await });

    view! {
        <IbisTitle key="create-article" />
        <h1 class="my-4 font-serif text-4xl font-bold">{move || tr!("create-article")}</h1>
        <Show
            when=move || create_response.get().is_some()
            fallback=move || {
                view! {
                    <SuspenseError result=instances>
                        {move || Suspend::new(async move {
                            let instances_ = instances.await;
                            view! {
                                <div class="item-view">
                                    <input
                                        class="w-full input input-primary"
                                        type="text"
                                        required
                                        placeholder="Title"
                                        bind:value=title
                                        prop:disabled=move || wait_for_response.get()
                                    />

                                    <label for="instance">"Instance: "</label>
                                    <select
                                        id="instance"
                                        class="select select-primary select-sm mt-4"
                                        bind:value=instance_id
                                        required
                                    >
                                        // Put local instance first to be the default
                                        {if let Ok(instances) = instances_ {
                                            instance_id.1.set(instances[0].instance.id.0.to_string());
                                            instances
                                                .into_iter()
                                                .map(|i| i.instance)
                                                .sorted_by(|a, b| Ord::cmp(&b.local, &a.local))
                                                .map(|i| {

                                                    view! { <option value=i.id.0>{i.domain}</option> }
                                                })
                                                .collect_view()
                                        } else {
                                            vec![]
                                        }}
                                    </select>

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
                                            bind:value=summary
                                            required
                                        />

                                        <button
                                            class="btn btn-primary"
                                            prop:disabled=move || create_button_is_disabled.get()
                                            on:click=move |_| {
                                                create_action
                                                    .dispatch((
                                                        title.0.get(),
                                                        content.get(),
                                                        summary.0.get(),
                                                        instance_id.0.get(),
                                                    ));
                                            }
                                        >
                                            Submit
                                        </button>
                                    </div>
                                </div>
                                <div class="divider"></div>
                                <div class="item-view">
                                    <h2 class="my-4 font-serif text-xl font-bold">
                                        Import from Wikipedia
                                    </h2>
                                    <div class=" flex flex-row">
                                        <input
                                            class="mr-4 input input-primary grow"
                                            type="text"
                                            placeholder="https://en.wikipedia.org/wiki/Animal"
                                            bind:value=import_url
                                            required
                                        />
                                        <button
                                            class="btn btn-primary"
                                            prop:disabled=move || import_button_is_disabled.get()
                                            on:click=move |_| {
                                                import_action.dispatch(import_url.0.get());
                                            }
                                        >
                                            Import
                                        </button>
                                    </div>
                                </div>
                            }
                        })}
                    </SuspenseError>
                }
            }
        >

            <Redirect path=article_path(
                &create_response.get().expect("response is defined here").article,
            ) />
        </Show>
    }
}
