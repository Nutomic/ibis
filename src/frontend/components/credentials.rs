use leptos::{ev::KeyboardEvent, prelude::*};

#[component]
pub fn CredentialsForm(
    title: &'static str,
    action_label: &'static str,
    action: Action<(String, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (password, set_password) = signal(String::new());
    let (username, set_username) = signal(String::new());

    let dispatch_action = move || action.dispatch((username.get(), password.get()));

    let button_is_disabled = Signal::derive(move || {
        disabled.get() || password.get().is_empty() || username.get().is_empty()
    });

    view! {
        <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>
            <h1 class="text-4xl font-bold font-serif my-4 grow max-w-fit">{title}</h1>
            {move || {
                error
                    .get()
                    .map(|err| {
                        view! { <p class="alert alert-error">{err}</p> }
                    })
            }}

            <input
                type="text"
                class="input input-primary input-bordered"
                required
                placeholder="Username"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: KeyboardEvent| {
                    let val = event_target_value(&ev);
                    set_username.update(|v| *v = val);
                }

                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_username.update(|v| *v = val);
                }
            />
            <div class="h-2"></div>
            <input
                type="password"
                class="input input-primary input-bordered"
                required
                placeholder="Password"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: KeyboardEvent| {
                    match &*ev.key() {
                        "Enter" => {
                            dispatch_action();
                        }
                        _ => {
                            let val = event_target_value(&ev);
                            set_password.update(|p| *p = val);
                        }
                    }
                }

                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_password.update(|p| *p = val);
                }
            />

            <div>
                <button
                    class="btn btn-primary my-2"
                    prop:disabled=move || button_is_disabled.get()
                    on:click=move |_| {
                        dispatch_action();
                    }
                >
                    {action_label}
                </button>
            </div>
        </form>
    }
}
