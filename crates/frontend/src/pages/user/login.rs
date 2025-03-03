use crate::utils::resources::site;
use ibis_api_client::errors::FrontendResultExt;
use ibis_api_client::{user::LoginUserParams, CLIENT};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::Redirect;

#[component]
pub fn Login() -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (username, set_username) = signal(String::new());
    let (login_response, set_login_response) = signal(false);
    let (wait_for_response, set_wait_for_response) = signal(false);

    let login_action = Action::new(move |(email, password): &(String, String)| {
        let username = email.to_string();
        let password = password.to_string();
        let params = LoginUserParams { username, password };
        async move {
            set_wait_for_response.update(|w| *w = true);
            CLIENT.login(params).await.error_popup(|_| {
                site().refetch();
                set_login_response.set(true);
            });
            set_wait_for_response.update(|w| *w = false);
        }
    });
    let dispatch_action = move || login_action.dispatch((username.get(), password.get()));

    let button_is_disabled = Signal::derive(move || {
        wait_for_response.get() || password.get().is_empty() || username.get().is_empty()
    });

    view! {
        <Title text="Login" />
        <Show
            when=move || login_response.get()
            fallback=move || {
                view! {
                    <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>
                        <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">Login</h1>

                        <input
                            type="text"
                            class="input input-primary input-bordered"
                            required
                            placeholder="Username"
                            bind:value=(username, set_username)
                            prop:disabled=move || wait_for_response.get()
                        />
                        <div class="h-2"></div>
                        <input
                            type="password"
                            class="input input-primary input-bordered"
                            required
                            placeholder="Password"
                            prop:disabled=move || wait_for_response.get()
                            bind:value=(password, set_password)
                        />

                        <div>
                            <button
                                class="my-2 btn btn-primary"
                                prop:disabled=move || button_is_disabled.get()
                                on:click=move |_| {
                                    dispatch_action();
                                }
                            >
                                Login
                            </button>
                        </div>
                    </form>
                }
            }
        >

            <Redirect path="/" />
        </Show>
    }
}
