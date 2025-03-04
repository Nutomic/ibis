use crate::{components::suspense_error::SuspenseError, utils::resources::site};
use ibis_api_client::{
    CLIENT,
    errors::FrontendResultExt,
    user::{ChangePasswordParams, UpdateUserParams},
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn UserEditProfile() -> impl IntoView {
    let (saved, set_saved) = signal(false);

    let submit_action = Action::new(move |params: &UpdateUserParams| {
        let params = params.clone();
        async move {
            CLIENT.update_user_profile(params).await.error_popup(|_| {
                set_saved.set(true);
                site().refetch();
            });
        }
    });
    let change_password_action = Action::new(move |params: &ChangePasswordParams| {
        let params = params.clone();
        async move {
            CLIENT.change_password(params).await.error_popup(|_| {
                set_saved.set(true);
                site().refetch();
            });
        }
    });
    let site = site();

    // TODO: It would make sense to use a table for the labels and inputs, but for some reason
    //       that completely breaks reactivity.
    view! {
        <Title text="Edit Profile" />
        <SuspenseError result=site>
            {Suspend::new(async move {
                site.await
                    .ok()
                    .and_then(|site| site.my_profile)
                    .map(|my_profile| {
                        let display_name = signal(
                            my_profile.person.display_name.clone().unwrap_or_default(),
                        );
                        let bio = signal(my_profile.person.bio.clone().unwrap_or_default());
                        let email = signal(my_profile.local_user.email.clone().unwrap_or_default());
                        let new_password = signal(String::new());
                        let confirm_new_password = signal(String::new());
                        let old_password = signal(String::new());
                        view! {
                            <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                                Edit Profile
                            </h1>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40" for="displayname">
                                    Displayname
                                </label>
                                <input
                                    type="text"
                                    id="displayname"
                                    class="w-80 input input-secondary input-bordered"
                                    bind:value=display_name
                                />
                            </div>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40" for="bio">
                                    "Bio (Markdown supported)"
                                </label>
                                <textarea
                                    id="bio"
                                    class="w-80 text-base textarea textarea-secondary"
                                    bind:value=bio
                                >
                                    bio.0.get()
                                </textarea>
                            </div>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40" for="email">
                                    Email
                                </label>
                                <input
                                    type="text"
                                    id="email"
                                    class="w-80 input input-secondary input-bordered"
                                    bind:value=email
                                />
                            </div>
                            <button
                                class="btn btn-primary"
                                on:click=move |_| {
                                    let form = UpdateUserParams {
                                        display_name: Some(display_name.0.get()),
                                        bio: Some(bio.0.get()),
                                        email: Some(email.0.get()),
                                    };
                                    submit_action.dispatch(form);
                                }
                            >
                                Submit
                            </button>

                            <div class="divider"></div>

                            <h2 class="flex-auto my-6 font-serif text-2xl font-bold grow">
                                Change Password
                            </h2>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40" for="new_password">
                                    New password
                                </label>
                                <input
                                    type="password"
                                    id="new_password"
                                    class="w-80 input input-secondary input-bordered"
                                    bind:value=new_password
                                />
                            </div>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40" for="confirm_new_password">
                                    Confirm new password
                                </label>
                                <input
                                    type="password"
                                    id="confirm_new_password"
                                    class="w-80 input input-secondary input-bordered"
                                    bind:value=confirm_new_password
                                />
                            </div>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40" for="old_password">
                                    Old password
                                </label>
                                <input
                                    type="password"
                                    id="old_password"
                                    class="w-80 input input-secondary input-bordered"
                                    bind:value=old_password
                                />
                            </div>
                            <button
                                class="btn btn-primary"
                                on:click=move |_| {
                                    let form = ChangePasswordParams {
                                        new_password: new_password.0.get(),
                                        confirm_new_password: confirm_new_password.0.get(),
                                        old_password: old_password.0.get(),
                                    };
                                    change_password_action.dispatch(form);
                                }
                            >
                                Save
                            </button>

                            <Show when=move || saved.get()>
                                <div class="toast">
                                    <div class="alert alert-info">
                                        <span>Saved!</span>
                                    </div>
                                </div>
                            </Show>
                        }
                    })
            })}

        </SuspenseError>
    }
}
