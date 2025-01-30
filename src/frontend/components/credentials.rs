use leptos::prelude::*;

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
            <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">{title}</h1>
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
                bind:value=(username, set_username)
                prop:disabled=move || disabled.get()
            />
            <div class="h-2"></div>
            <input
                type="password"
                class="input input-primary input-bordered"
                required
                placeholder="Password"
                prop:disabled=move || disabled.get()
                bind:value=(password, set_password)
            />

            <div>
                <button
                    class="my-2 btn btn-primary"
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
