use crate::frontend::app::GlobalState;
use leptos::*;
use leptos::{component, use_context, view, IntoView, RwSignal, SignalWith};
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    let logout_action = create_action(move |_| async move {
        GlobalState::api_client().logout().await.unwrap();
        expect_context::<RwSignal<GlobalState>>()
            .get_untracked()
            .update_my_profile();
    });
    view! {
        <nav class="inner">
            <li>
                <A href="/">"Main Page"</A>
            </li>
            <Show
                when=move || global_state.with(|state| state.my_profile.is_none())
                fallback=move || {
                    view! {
                        <p>"Logged in as: "
                            {
                                move || global_state.with(|state| state.my_profile.clone().unwrap().person.username)
                            }
                            <button on:click=move |_| logout_action.dispatch(())>
                                Logout
                            </button>
                        </p>
                    }
                }
            >
            <li>
                <A href="/login">"Login"</A>
            </li>
            <li>
                <A href="/register">"Register"</A>
            </li>
        </Show>
        </nav>
    }
}
