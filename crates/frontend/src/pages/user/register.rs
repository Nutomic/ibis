use crate::{components::suspense_error::SuspenseError, utils::resources::site};
use ibis_api_client::{
    CLIENT,
    errors::FrontendResultExt,
    user::{RegisterUserParams, RegistrationResponse},
};
use leptos::prelude::*;
use leptos_meta::Title;
use log::info;

#[component]
pub fn Register() -> impl IntoView {
    let username = signal(String::new());
    let email = signal(String::new());
    let password = signal(String::new());
    let confirm_password = signal(String::new());
    let (register_response, set_register_response) = signal(None::<RegistrationResponse>);
    let (loading, set_loading) = signal(false);

    let register_action = Action::new(move |(): &()| {
        let params = RegisterUserParams {
            username: username.0.get().to_string(),
            email: Some(email.0.get().to_string()),
            password: password.0.get().to_string(),
            confirm_password: confirm_password.0.get().to_string(),
        };
        info!("Try to register new account for {}", params.username);
        async move {
            set_loading.set(true);
            CLIENT.register(params).await.error_popup(|res| {
                site().refetch();
                set_register_response.set(Some(res));
            });
            set_loading.set(false);
        }
    });

    let dispatch_action = move || register_action.dispatch(());

    let site = site();

    view! {
        <Title text="Register" />
        <SuspenseError result=site>
            {move || Suspend::new(async move {
                let email_required = site
                    .await
                    .map(|s| s.config.email_required)
                    .unwrap_or_default();
                let email_placeholder = if email_required { "Email" } else { "Email (optional)" };
                let button_is_disabled = Signal::derive(move || {
                    let disabled = loading.get() || username.0.get().is_empty()
                        || password.0.get().is_empty() || confirm_password.0.get().is_empty();
                    if email_required && email.0.get().is_empty() {
                        return false;
                    }
                    disabled
                });
                view! {
                    <Show
                        when=move || register_response.get().is_some()
                        fallback=move || {
                            view! {
                                <Show
                                    when=move || {
                                        register_response
                                            .get()
                                            .map(|r| r.email_verification_required)
                                            .unwrap_or_default()
                                    }
                                    fallback=|| {
                                        view! { <p>"You have successfully registered."</p> }
                                    }
                                >
                                    <p>
                                        "Registration successful, now verify the email address to login"
                                    </p>
                                </Show>
                            }
                        }
                    >
                        <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>
                            <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">
                                Register
                            </h1>

                            <input
                                type="text"
                                class="input input-primary input-bordered my-1"
                                required
                                placeholder="Username"
                                bind:value=username
                                prop:disabled=move || loading.get()
                            />
                            <input
                                type="text"
                                class="input input-primary input-bordered my-1"
                                required
                                placeholder=email_placeholder
                                bind:value=email
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
                            <input
                                type="password"
                                class="input input-primary input-bordered my-1"
                                required
                                placeholder="Confirm password"
                                prop:disabled=move || loading.get()
                                bind:value=confirm_password
                            />

                            <div>
                                <button
                                    class="my-2 btn btn-primary"
                                    prop:disabled=move || button_is_disabled.get()
                                    on:click=move |_| {
                                        dispatch_action();
                                    }
                                >
                                    Register
                                </button>
                            </div>
                        </form>
                    </Show>
                }
            })}
        </SuspenseError>
    }
}
