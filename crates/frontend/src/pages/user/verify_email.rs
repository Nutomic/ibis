use ibis_api_client::{CLIENT, errors::FrontendResultExt};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

#[component]
pub fn VerifyEmail() -> impl IntoView {
    let success = signal(false);
    let verify_email_action = Action::new(move |token: &String| {
        let token = token.to_string();
        async move {
            CLIENT
                .verify_email(token)
                .await
                .error_popup(|_| success.1.set(true));
        }
    });
    use_query_map().with(|params| {
        if let Some(token) = params.get("token") {
            verify_email_action.dispatch(token);
        }
    });
    view! {
        <Show when=move || verify_email_action.pending().get()>Loading...</Show>
        <Show when=move || success.0.get()>"Email successfully verified. You can login now."</Show>
    }
}
