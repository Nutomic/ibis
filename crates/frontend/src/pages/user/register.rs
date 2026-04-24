use ibis_api_client::{
    CLIENT,
    errors::FrontendResultExt,
    user::{RegisterUserParams, RegistrationResponse},
};
use ibis_frontend_components::{suspense_error::SuspenseError, utils::resources::site};
use leptos::{html::Textarea, prelude::*};
use leptos_meta::Title;
use leptos_use::{UseTextareaAutosizeReturn, use_textarea_autosize};
use log::info;

#[component]
pub fn Register() -> impl IntoView {
    let username = signal(String::new());
    let email = signal(String::new());
    let password = signal(String::new());
    let confirm_password = signal(String::new());
    let (registration_application, set_registration_application) = signal(String::new());
    let textarea_ref = NodeRef::<Textarea>::new();
    let UseTextareaAutosizeReturn {
        content: registration_application,
        set_content: set_registration_application,
        trigger_resize: _,
    } = use_textarea_autosize(textarea_ref);
    let (register_response, set_register_response) = signal(None::<RegistrationResponse>);
    let (loading, set_loading) = signal(false);

    let register_action = Action::new(move |(): &()| {
        let params = RegisterUserParams {
            username: username.0.get().clone(),
            email: Some(email.0.get().clone()),
            password: password.0.get().clone(),
            confirm_password: confirm_password.0.get().clone(),
            registration_application: Some(registration_application.get().clone())
                .filter(|s| !s.is_empty()),
        };
        info!("Try to register new account for {}", params.username);
        async move {
            set_loading.set(true);
            CLIENT.register(params).await.error_popup(|res| {
                site().refetch();
                set_register_response.set(Some(res));
            });
            set_loading.set(false);
        }
    });

    let dispatch_action = move || register_action.dispatch(());

    let site = site();

    view! {
        <Title text="Register" />
        <SuspenseError result=site>
            {move || Suspend::new(async move {
                let config = site.await.map(|s| s.config).unwrap_or_default();
                let email_required = config.email_required;
                let registration_question_is_some = config.registration_question.is_some();
                let registration_question = config.registration_question;
                let email_placeholder = if email_required { "Email" } else { "Email (optional)" };
                let button_is_disabled = Signal::derive(move || {
                    info!("{}", registration_application.get());
                    let disabled = loading.get() || username.0.get().is_empty()
                        || password.0.get().is_empty() || confirm_password.0.get().is_empty()
                        || (registration_question_is_some
                            && registration_application.get().is_empty());
                    if email_required && email.0.get().is_empty() {
                        return false;
                    }
                    disabled
                });
                view! {
                    <Show
                        when=move || register_response.get().is_none()
                        fallback=move || {
                            let email_verify = register_response
                                .get()
                                .map(|r| r.email_verification_required)
                                .unwrap_or_default();
                            let admin_review_required = register_response
                                .get()
                                .map(|r| r.admin_review_required)
                                .unwrap_or_default();
                            view! {
                                <p>
                                    <Show when=move || {
                                        email_verify
                                    }>

                                        "Registration successful, now verify the email address to login."
                                    </Show>
                                    <Show when=move || {
                                        admin_review_required
                                    }>
                                        "Registration successful, now wait for admin approval. You will receive an email once approved."
                                    </Show>
                                    <Show when=move || {
                                        !email_verify && !admin_review_required
                                    }>"You have successfully registered."</Show>
                                </p>
                            }
                        }
                    >
                        <form class="form-control max-w-80" on:submit=|ev| ev.prevent_default()>
                            <h1 class="my-4 font-serif text-4xl font-bold grow max-w-fit">
                                Register
                            </h1>

                            <input
                                type="text"
                                class="input input-primary input-bordered my-1"
                                required
                                placeholder="Username"
                                bind:value=username
                                prop:disabled=move || loading.get()
                            />
                            <input
                                type="text"
                                class="input input-primary input-bordered my-1"
                                required
                                placeholder=email_placeholder
                                bind:value=email
                                prop:disabled=move || loading.get()
                            />
                            <input
                                type="password"
                                class="input input-primary input-bordered my-1"
                                required
                                placeholder="Password"
                                prop:disabled=move || loading.get()
                                bind:value=password
                            />
                            <input
                                type="password"
                                class="input input-primary input-bordered my-1"
                                required
                                placeholder="Confirm password"
                                prop:disabled=move || loading.get()
                                bind:value=confirm_password
                            />
                            {registration_question
                                .clone()
                                .map(|r| {
                                    view! {
                                        <div>{r}</div>
                                        <textarea
                                            class="input input-primary input-bordered my-1 min-h-16"
                                            required
                                            prop:disabled=move || loading.get()
                                            prop:value=registration_application
                                            on:input=move |evt| {
                                                let val = event_target_value(&evt);
                                                set_registration_application.set(val);
                                            }
                                            node_ref=textarea_ref
                                        />
                                    }
                                })}

                            <div>
                                <button
                                    class="my-2 btn btn-primary"
                                    prop:disabled=move || button_is_disabled.get()
                                    on:click=move |_| {
                                        dispatch_action();
                                    }
                                >
                                    Register
                                </button>
                            </div>
                        </form>
                    </Show>
                }
            })}
        </SuspenseError>
    }
}
