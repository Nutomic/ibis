use crate::frontend::{
    api::CLIENT,
    components::connect::ConnectView,
    instance_title_with_domain,
    instance_updated,
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn ListInstances() -> impl IntoView {
    let instances = Resource::new(
        move || (),
        |_| async move { CLIENT.list_instances().await.unwrap() },
    );

    view! {
        <Title text="Instances" />
        <h1 class="my-4 font-serif text-4xl font-bold">Instances</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <Show
                when=move || { !instances.get().unwrap_or_default().is_empty() }
                fallback=move || view! { <ConnectView res=instances /> }
            >
                <ul class="my-4 list-none">
                    {move || {
                        instances
                            .get()
                            .map(|a| {
                                a.into_iter()
                                    .map(|ref i| {
                                        view! {
                                            <li>
                                                <div class="m-4 shadow card bg-base-100">
                                                    <div class="p-4 card-body">
                                                        <div class="flex">
                                                            <a
                                                                class="card-title grow"
                                                                href=format!("/instance/{}", i.domain)
                                                            >
                                                                {instance_title_with_domain(i)}
                                                            </a>
                                                            {instance_updated(i)}
                                                        </div>
                                                        <p>{i.topic.clone()}</p>
                                                    </div>
                                                </div>
                                            </li>
                                        }
                                    })
                                    .collect::<Vec<_>>()
                            })
                    }}

                </ul>
            </Show>
        </Suspense>
    }
}
