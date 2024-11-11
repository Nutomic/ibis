use crate::frontend::{app::GlobalState, components::connect::ConnectView};
use leptos::*;

#[component]
pub fn ListInstances() -> impl IntoView {
    let instances = create_resource(
        move || (),
        |_| async move { GlobalState::api_client().list_instances().await.unwrap() },
    );

    view! {
        <h1 class="text-4xl font-bold font-serif my-4">Instances</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <Show
                when=move || { !instances.get().unwrap_or_default().is_empty() }
                fallback=move || view! { <ConnectView res=instances /> }
            >
                <ul class="list-none my-4">
                    {move || {
                        instances
                            .get()
                            .map(|a| {
                                a.into_iter()
                                    .map(|i| {
                                        view! {
                                            <li>
                                                <a
                                                    class="link text-lg"
                                                    href=format!("/instance/{}", i.domain)
                                                >
                                                    {i.domain}
                                                </a>
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
