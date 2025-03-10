use crate::{components::oauth_login_button::OauthLoginButtons, utils::resources::site};
use ibis_api_client::{CLIENT, errors::FrontendResultExt, user::LoginUserParams};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::Redirect;

#[component]
pub fn Login() -> impl IntoView {
    let password = signal(String::new());
    let username_or_email = signal(String::new());
    let (login_response, set_login_response) = signal(false);
    let (loading, set_loading) = signal(false);

    let login_action = Action::new(move |(): &()| {
        let username_or_email = username_or_email.0.get().to_string();
        let password = password.0.get().to_string();
        let params = LoginUserParams {
            username_or_email,
            password,
        };
        async move {
            set_loading.set(true);
            CLIENT.login(params).await.error_popup(|_| {
                site().refetch();
                set_login_response.set(true);
            });
            set_loading.set(false);
        }
    });
    let dispatch_action = move || login_action.dispatch(());

    let button_is_disabled = Signal::derive(move || {
        loading.get() || password.0.get().is_empty() || username_or_email.0.get().is_empty()
    });

    view! {
        <Title text="Login" />
        <Show
            when=move || !login_response.get()
            fallback=move || {
                view! { <Redirect path="/" /> }
            }
        >
            <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>
                <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">Login</h1>

                <input
                    type="text"
                    class="input input-primary input-bordered my-1"
                    required
                    placeholder="Username or email"
                    bind:value=username_or_email
                    prop:disabled=move || loading.get()
                />
                <input
                    type="password"
                    class="input input-primary input-bordered my-1"
                    required
                    placeholder="Password"
                    prop:disabled=move || loading.get()
                    bind:value=password
                />
                <a href="/account/request_password_reset" class="link text-sm">
                    Reset password
                </a>

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

            <OauthLoginButtons username=username_or_email.0 />
        </Show>
    }
}
