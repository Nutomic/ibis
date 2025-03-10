use ibis_api_client::{CLIENT, errors::FrontendResultExt, user::ChangePasswordAfterReset};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

#[component]
pub fn ResetPassword() -> impl IntoView {
    let password = signal(String::new());
    let confirm_password = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (response_received, set_response_received) = signal(false);
    let query_map = use_query_map().get();
    let token = Signal::derive(move || query_map.get("token"));

    let login_action = Action::new(move |(): &()| {
        let password = password.0.get().to_string();
        let confirm_password = confirm_password.0.get().to_string();
        let params = ChangePasswordAfterReset {
            password,
            confirm_password,
            token: token.clone().get().expect("has token"),
        };
        async move {
            set_loading.set(true);
            CLIENT
                .change_password_after_reset(params)
                .await
                .error_popup(|_| {
                    set_response_received.set(true);
                });
            set_loading.set(false);
        }
    });
    let dispatch_action = move || login_action.dispatch(());

    let button_is_disabled = Signal::derive(move || {
        loading.get() || password.0.get().is_empty() || confirm_password.0.get().is_empty()
    });

    view! {
        <Title text="Set new password" />
        <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">Reset password</h1>
        <Show when=move || token.get().is_some() fallback=move || view! { Missing token }>
            <Show
                when=move || !response_received.get()
                fallback=move || view! { "Password changed, you can login now" }
            >
                <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>

                    <input
                        type="password"
                        class="input input-primary input-bordered my-1"
                        required
                        placeholder="Password"
                        bind:value=password
                        prop:disabled=move || loading.get()
                    />
                    <input
                        type="password"
                        class="input input-primary input-bordered my-1"
                        required
                        placeholder="Confirm password"
                        bind:value=confirm_password
                        prop:disabled=move || loading.get()
                    />

                    <div>
                        <button
                            class="my-2 btn btn-primary"
                            prop:disabled=move || button_is_disabled
                            on:click=move |_| {
                                dispatch_action();
                            }
                        >
                            Set new password
                        </button>
                    </div>
                </form>
            </Show>
        </Show>
    }
}
