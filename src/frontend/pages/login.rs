use crate::{
    common::LoginUserForm,
    frontend::{app::GlobalState, components::credentials::*},
};
use leptos::*;
use leptos_router::Redirect;

#[component]
pub fn Login() -> impl IntoView {
    let (login_response, set_login_response) = create_signal(None::<()>);
    let (login_error, set_login_error) = create_signal(None::<String>);
    let (wait_for_response, set_wait_for_response) = create_signal(false);

    let login_action = create_action(move |(email, password): &(String, String)| {
        let username = email.to_string();
        let password = password.to_string();
        let credentials = LoginUserForm { username, password };
        async move {
            set_wait_for_response.update(|w| *w = true);
            let result = GlobalState::api_client().login(credentials).await;
            set_wait_for_response.update(|w| *w = false);
            match result {
                Ok(res) => {
                    expect_context::<RwSignal<GlobalState>>()
                        .update(|state| state.my_profile = Some(res));
                    set_login_response.update(|v| *v = Some(()));
                    set_login_error.update(|e| *e = None);
                }
                Err(err) => {
                    let msg = err.0.to_string();
                    log::warn!("Unable to login: {msg}");
                    set_login_error.update(|e| *e = Some(msg));
                }
            }
        }
    });

    let disabled = Signal::derive(move || wait_for_response.get());

    view! {
        <Show
            when=move || login_response.get().is_some()
            fallback=move || {
                view! {
                    <CredentialsForm
                        title="Please enter the desired credentials"
                        action_label="Login"
                        action=login_action
                        error=login_error.into()
                        disabled
                    />
                }
            }
        >
            <Redirect path="/"/>
        </Show>
    }
}
