use crate::{
    common::instance::GetInstanceParams,
    frontend::{
        api::CLIENT,
        components::suspense_error::SuspenseError,
        utils::{formatting::user_link, resources::site},
    },
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn About() -> impl IntoView {
    let site = site();
    let instance = Resource::new(
        || (),
        |_| async move {
            CLIENT
                .get_instance(&GetInstanceParams {
                    id: None,
                    hostname: None,
                })
                .await
        },
    );
    view! {
        <Title text="About" />
        <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">About</h1>
        <SuspenseError result=instance>
            {move || Suspend::new(async move {
                let site = site.await;
                let instance = instance.await;
                instance
                    .ok()
                    .and_then(|instance| {
                        site.ok()
                            .map(|site| {
                                view! {
                                    <div>"Topic: "{instance.instance.topic}</div>
                                    <div>"Administrated by: "{user_link(&site.admin)}</div>
                                }
                            })
                    })
            })}
        </SuspenseError>
    }
}
