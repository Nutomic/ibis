use crate::frontend::app::GlobalState;
use leptos::*;
use url::Url;

#[component]
pub fn ListInstances() -> impl IntoView {
    let instances = create_resource(
        move || (),
        |_| async move { GlobalState::api_client().list_instances().await.unwrap() },
    );

    let connect_ibis_wiki = create_action(move |_: &()| async move {
        GlobalState::api_client()
            .resolve_instance(Url::parse("https://ibis.wiki").unwrap())
            .await
            .unwrap();
        instances.refetch();
    });
    let fallback = move || {
        view! {
            <div class="flex justify-center h-screen">
                <button
                    class="btn btn-primary place-self-center"
                    on:click=move |_| connect_ibis_wiki.dispatch(())
                >
                    Connect with ibis.wiki
                </button>
            </div>
        }
    };

    view! {
        <h1 class="text-4xl font-bold font-serif my-4">Instances</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <Show
                when=move || { !instances.get().unwrap_or_default().is_empty() }
                fallback=fallback
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
