use crate::frontend::api::logout;
use crate::frontend::app::GlobalState;
use leptos::*;
use leptos::{component, use_context, view, IntoView, RwSignal, SignalWith};
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    let global_state = use_context::<RwSignal<GlobalState>>().unwrap();
    // TODO: use `<Show when` based on auth token for login/register/logout
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
                            <button on:click=move |_| {
                                // TODO: not executed
                                dbg!(1);
                                do_logout()
                            }>
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

fn do_logout() {
    dbg!("do logout");
    create_action(move |()| async move {
        dbg!("run logout action");
        logout(&GlobalState::read_hostname()).await.unwrap();
        expect_context::<RwSignal<GlobalState>>()
            .get()
            .update_my_profile();
    });
}
