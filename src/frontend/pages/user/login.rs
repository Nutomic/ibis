use crate::{
    common::user::LoginUserParams,
    frontend::{api::CLIENT, components::credentials::*, utils::resources::site},
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::components::Redirect;

#[component]
pub fn Login() -> impl IntoView {
    let (login_response, set_login_response) = signal(false);
    let (login_error, set_login_error) = signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = signal(false);

    let login_action = Action::new(move |(email, password): &(String, String)| {
        let username = email.to_string();
        let password = password.to_string();
        let params = LoginUserParams { username, password };
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = CLIENT.login(params).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(_res) => {
                    site().refetch();
                    set_login_response.set(true);
                    set_login_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = err.to_string();
                    log::warn!("Unable to login: {msg}");
                    set_login_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
        <Title text="Login" />
        <Show
            when=move || login_response.get()
            fallback=move || {
                view! {
                    <CredentialsForm
                        title="Login"
                        action_label="Login"
                        action=login_action
                        error=login_error.into()
                        disabled
                    />
                }
            }
        >

            <Redirect path="/" />
        </Show>
    }
}
