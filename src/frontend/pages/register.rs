use crate::{
    common::{LocalUserView, RegisterUserForm},
    frontend::{api::CLIENT, app::site, components::credentials::*, error::MyResult},
};
use leptos::{logging::log, *};

#[component]
pub fn Register() -> impl IntoView {
    let (register_response, set_register_response) = create_signal(None::<()>);
    let (register_error, set_register_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);

    let register_action = create_action(move |(email, password): &(String, String)| {
        let username = email.to_string();
        let password = password.to_string();
        let credentials = RegisterUserForm { username, password };
        log!("Try to register new account for {}", credentials.username);
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result: MyResult<LocalUserView> = CLIENT.register(credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(_res) => {
                    site().refetch();
                    set_register_response.update(|v| *v = Some(()));
                    set_register_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = err.0.to_string();
                    log::warn!("Unable to register new account: {msg}");
                    set_register_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
        <Show
            when=move || register_response.get().is_some()
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
