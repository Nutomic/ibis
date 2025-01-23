use crate::{common::instance::UpdateInstanceParams, frontend::api::CLIENT};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn InstanceSettings() -> impl IntoView {
    let (saved, set_saved) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);
    let instance = Resource::new(
        || (),
        |_| async move { CLIENT.get_local_instance().await.unwrap() },
    );

    let submit_action = Action::new(move |params: &UpdateInstanceParams| {
        let params = params.clone();
        async move {
            let result = CLIENT.update_local_instance(&params).await;
            match result {
                Ok(_res) => {
                    instance.refetch();
                    set_saved.set(true);
                    set_submit_error.set(None);
                }
                Err(err) => {
                    let msg = err.to_string();
                    log::warn!("Unable to update profile: {msg}");
                    set_submit_error.set(Some(msg));
                }
            }
        }
    });

    // TODO: It would make sense to use a table for the labels and inputs, but for some reason
    //       that completely breaks reactivity.
    view! {
        <Title text="Instance Settings" />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || Suspend::new(async move {
                let instance = instance.await;
                let (name, set_name) = signal(instance.instance.name.unwrap_or_default());
                let (topic, set_topic) = signal(instance.instance.topic.unwrap_or_default());
                view! {
                    <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                        "Instance Settings"
                    </h1>
                    {move || {
                        submit_error
                            .get()
                            .map(|err| {
                                view! { <p class="alert alert-error">{err}</p> }
                            })
                    }}
                    <div class="flex flex-row mb-2">
                        <label class="block w-20" for="name">
                            Name
                        </label>
                        <input
                            type="text"
                            id="name"
                            class="w-80 input input-secondary input-bordered"
                            prop:value=name
                            value=name
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_name.set(val);
                            }
                        />
                    </div>
                    <div class="flex flex-row mb-2">
                        <label class="block w-20" for="topic">
                            "Topic"
                        </label>
                        <input
                            type="text"
                            id="name"
                            class="w-80 input input-secondary input-bordered"
                            prop:value=topic
                            value=topic
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_topic.set(val);
                            }
                        />
                    </div>
                    <button
                        class="btn btn-primary"
                        on:click=move |_| {
                            let form = UpdateInstanceParams {
                                name: Some(name.get()),
                                topic: Some(topic.get()),
                            };
                            submit_action.dispatch(form);
                        }
                    >
                        Submit
                    </button>

                    <Show when=move || saved.get()>
                        <div class="toast">
                            <div class="alert alert-info">
                                <span>Saved!</span>
                            </div>
                        </div>
                    </Show>
                }
            })}

        </Suspense>
    }
}
