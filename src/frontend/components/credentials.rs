use leptos::{ev, *};

#[component]
pub fn CredentialsForm(
    title: &'static str,
    action_label: &'static str,
    action: Action<(String, String), ()>,
    error: Signal<Option<String>>,
    disabled: Signal<bool>,
) -> impl IntoView {
    let (password, set_password) = create_signal(String::new());
    let (username, set_username) = create_signal(String::new());

    let dispatch_action = move || action.dispatch((username.get(), password.get()));

    let button_is_disabled = Signal::derive(move || {
        disabled.get() || password.get().is_empty() || username.get().is_empty()
    });

    view! {
        <form on:submit=|ev| ev.prevent_default()>
            <p>{title}</p>
            {move || {
                error
                    .get()
                    .map(|err| {
                        view! { <p style="color:red;">{err}</p> }
                    })
            }}
            <input
                type="text"
                required
                placeholder="Username"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: ev::KeyboardEvent| {
                    let val = event_target_value(&ev);
                    set_username.update(|v| *v = val);
                }
                on:change=move |ev| {
                    let val = event_target_value(&ev);
                    set_username.update(|v| *v = val);
                }
            />
            <input
                type="password"
                required
                placeholder="Password"
                prop:disabled=move || disabled.get()
                on:keyup=move |ev: ev::KeyboardEvent| {
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
                    prop:disabled=move || button_is_disabled.get()
                    on:click=move |_| dispatch_action()>
                    {action_label}
                </button>
            </div>
        </form>
    }
}
