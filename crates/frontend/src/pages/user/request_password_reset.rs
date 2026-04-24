use ibis_api_client::{CLIENT, errors::FrontendResultExt, user::PasswordReset};
use ibis_frontend_components::utils::i18n::IbisTitle;
use leptos::prelude::*;
use leptos_fluent::tr;

#[component]
pub fn RequestPasswordReset() -> impl IntoView {
    let email = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (response_received, set_response_received) = signal(false);

    let reset_action = Action::new(move |(): &()| {
        let email = email.0.get().clone();
        let params = PasswordReset { email };
        async move {
            set_loading.set(true);
            CLIENT
                .request_password_reset(params)
                .await
                .error_popup(|_| {
                    set_response_received.set(true);
                });
            set_loading.set(false);
        }
    });
    let dispatch = move || reset_action.dispatch(());

    let button_is_disabled = Signal::derive(move || loading.get() || email.0.get().is_empty());

    view! {
        <IbisTitle key="reset-password" />
        <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">Reset password</h1>
        <Show
            when=move || !response_received.get()
            fallback=move || {
                view! { {tr!("check-email-for-reset-link")} }
            }
        >
            <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>

                <input
                    type="text"
                    class="input input-primary input-bordered my-1"
                    required
                    placeholder=tr!("username-or-email")
                    bind:value=email
                    prop:disabled=move || loading.get()
                />

                <div>
                    <button
                        class="my-2 btn btn-primary"
                        prop:disabled=move || button_is_disabled
                        on:click=move |_| {
                            dispatch();
                        }
                    >
                        {tr!("request-password-reset")}
                    </button>
                </div>
            </form>
        </Show>
    }
}
