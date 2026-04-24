use ibis_api_client::{CLIENT, errors::FrontendResultExt, user::LoginUserParams};
use ibis_frontend_components::{oauth_login_button::OauthLoginButtons, utils::{i18n::IbisTitle, resources::site}};
use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_router::components::Redirect;

#[component]
pub fn Login() -> impl IntoView {
    let password = signal(String::new());
    let username_or_email = signal(String::new());
    let (login_response, set_login_response) = signal(false);
    let (loading, set_loading) = signal(false);

    let login_action = Action::new(move |(): &()| {
        let username_or_email = username_or_email.0.get().clone();
        let password = password.0.get().clone();
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
        <IbisTitle key="login" />
        <Show
            when=move || !login_response.get()
            fallback=move || {
                view! { <Redirect path="/" /> }
            }
        >
            <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>
                <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">{tr!("login")}</h1>

                <input
                    type="text"
                    class="input input-primary input-bordered my-1"
                    required
                    placeholder=tr!("username-or-email")
                    bind:value=username_or_email
                    prop:disabled=move || loading.get()
                />
                <input
                    type="password"
                    class="input input-primary input-bordered my-1"
                    required
                    placeholder=tr!("password")
                    prop:disabled=move || loading.get()
                    bind:value=password
                />
                <a href="/account/request_password_reset" class="link text-sm">
                    {tr!("reset-password")}
                </a>

                <div>
                    <button
                        class="my-2 btn btn-primary"
                        prop:disabled=move || button_is_disabled.get()
                        on:click=move |_| {
                            dispatch_action();
                        }
                    >
                        {tr!("login")}
                    </button>
                </div>
            </form>

            <OauthLoginButtons username=username_or_email.0 />
        </Show>
    }
}
