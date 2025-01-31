use crate::{
    common::instance::DbInstance,
    frontend::{
        api::CLIENT,
        components::{connect::ConnectView, suspense_error::SuspenseError},
        utils::formatting::{instance_title_with_domain, instance_updated},
    },
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn Explore() -> impl IntoView {
    let instances = Resource::new(move || (), |_| async move { CLIENT.list_instances().await });

    view! {
        <Title text="Explore" />
        <h1 class="my-4 font-serif text-4xl font-bold">Instances</h1>
        <SuspenseError result=instances>
            {move || Suspend::new(async move {
                let instances_ = instances.await;
                let is_empty = instances_.as_ref().map(|i| i.is_empty()).unwrap_or(true);
                view! {
                    <Show
                        when=move || !is_empty
                        fallback=move || view! { <ConnectView res=instances /> }
                    >
                        <ul class="my-4 list-none">
                            {instances_
                                .clone()
                                .ok()
                                .iter()
                                .flatten()
                                .map(instance_card)
                                .collect::<Vec<_>>()}
                        </ul>
                    </Show>
                }
            })}
        </SuspenseError>
    }
}

pub fn instance_card(i: &DbInstance) -> impl IntoView {
    view! {
        <li>
            <div class="m-4 shadow card bg-base-100">
                <div class="p-4 card-body">
                    <div class="flex">
                        <a class="card-title grow" href=format!("/instance/{}", i.domain)>
                            {instance_title_with_domain(i)}
                        </a>
                        {instance_updated(i)}
                    </div>
                    <p>{i.topic.clone()}</p>
                </div>
            </div>
        </li>
    }
}
