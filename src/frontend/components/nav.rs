use crate::frontend::app::GlobalState;
use leptos::{component, use_context, view, IntoView, RwSignal, SignalWith, *};
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    let logout_action = create_action(move |_| async move {
        GlobalState::api_client().logout().await.unwrap();
        GlobalState::update_my_profile();
    });
    let registration_open = create_local_resource(
        || (),
        move |_| async move {
            GlobalState::api_client()
                .get_local_instance()
                .await
                .map(|i| i.registration_open)
                .unwrap_or_default()
        },
    );

    let (search_query, set_search_query) = create_signal(String::new());
    view! {
        <nav class="menu lg:menu-vertical lg:w-40">
            <li>
                <A href="/">"Main Page"</A>
            </li>
            <li>
                <A href="/article/list">"List Articles"</A>
            </li>
            <Show when=move || global_state.with(|state| state.my_profile.is_some())>
                <li>
                    <A href="/article/create">"Create Article"</A>
                </li>
                <li>
                    <A href="/conflicts">"Edit Conflicts"</A>
                </li>
            </Show>
            <li>
                <form 
                class="form-control m-0 p-1"
                on:submit=move |ev| {
                    ev.prevent_default();
                    let navigate = leptos_router::use_navigate();
                    let query = search_query.get();
                    if !query.is_empty() {
                        navigate(&format!("/search?query={query}"), Default::default());
                    }
                }>
                    <input
                        type="text"
                        class="input input-secondary input-bordered input-xs w-full rounded"
                        placeholder="Search"
                        prop:value=search_query
                        on:keyup=move |ev: ev::KeyboardEvent| {
                            let val = event_target_value(&ev);
                            set_search_query.update(|v| *v = val);
                        }
                    />

                    <button class="btn btn-xs btn-secondary">Go</button>
                </form>
            </li>
            <div class="divider"></div>
            <Show
                when=move || global_state.with(|state| state.my_profile.is_some())
                fallback=move || {
                    view! {
                        <li>
                            <A href="/login">"Login"</A>
                        </li>
                        <Show when=move || registration_open.get().unwrap_or_default()>
                            <li>
                                <A href="/register">"Register"</A>
                            </li>
                        </Show>
                    }
                }
            >

                {
                    let my_profile = global_state.with(|state| state.my_profile.clone().unwrap());
                    let profile_link = format!("/user/{}", my_profile.person.username);
                    view! {
                        <p class="self-center pb-2">
                            "Logged in as "
                            <a class="link"
                                href=profile_link
                            >
                                {my_profile.person.username}
                            </a>
                        </p>
                        <button class="btn" on:click=move |_| logout_action.dispatch(())>Logout</button>
                    }
                }

            </Show>
        </nav>
    }
}
