use crate::{components::credentials::*, utils::resources::site};
use ibis_api_client::{CLIENT, user::RegisterUserParams};
use leptos::prelude::*;
use leptos_meta::Title;
use log::info;

#[component]
pub fn Register() -> impl IntoView {
    let (register_response, set_register_response) = signal(false);
    let (register_error, set_register_error) = signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = signal(false);

    let register_action = Action::new(move |(email, password): &(String, String)| {
        let username = email.to_string();
        let password = password.to_string();
        let params = RegisterUserParams { username, password };
        info!("Try to register new account for {}", params.username);
        async move {
            set_wait_for_response.set(true);
            let result = CLIENT.register(params).await;
            set_wait_for_response.set(false);
            match result {
                Ok(_res) => {
                    site().refetch();
                    set_register_response.set(true);
                    set_register_error.set(None);
                }
                Err(err) => {
                    let msg = err.to_string();
                    log::warn!("Unable to register new account: {msg}");
                    set_register_error.set(Some(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
        <Title text="Register" />
        <Show
            when=move || register_response.get()
            fallback=move || {
                view! {
                    <CredentialsForm
                        title="Register"
                        action_label="Register"
                        action=register_action
                        error=register_error.into()
                        disabled
                    />
                }
            }
        >

            <p>"You have successfully registered."</p>
        </Show>
    }
}
