use crate::{
    common::instance::UpdateInstanceParams,
    frontend::{
        api::CLIENT,
        components::suspense_error::SuspenseError,
        utils::errors::FrontendResultExt,
    },
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn InstanceSettings() -> impl IntoView {
    let (saved, set_saved) = signal(false);
    let instance = Resource::new(|| (), |_| async move { CLIENT.get_local_instance().await });

    let submit_action = Action::new(move |params: &UpdateInstanceParams| {
        let params = params.clone();
        async move {
            CLIENT
                .update_local_instance(&params)
                .await
                .error_popup(|_| {
                    instance.refetch();
                    set_saved.set(true);
                });
        }
    });

    // TODO: It would make sense to use a table for the labels and inputs, but for some reason
    //       that completely breaks reactivity.
    view! {
        <Title text="Instance Settings" />
        <SuspenseError result=instance>
            {move || Suspend::new(async move {
                instance
                    .await
                    .map(|instance| {
                        let (name, set_name) = signal(instance.instance.name.unwrap_or_default());
                        let (topic, set_topic) = signal(
                            instance.instance.topic.unwrap_or_default(),
                        );
                        view! {
                            <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                                "Instance Settings"
                            </h1>
                            <div class="flex flex-row mb-2">
                                <label class="block w-20" for="name">
                                    Name
                                </label>
                                <input
                                    type="text"
                                    id="name"
                                    class="w-80 input input-secondary input-bordered"
                                    bind:value=(name, set_name)
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
                                    bind:value=(topic, set_topic)
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
                    })
            })}

        </SuspenseError>
    }
}
