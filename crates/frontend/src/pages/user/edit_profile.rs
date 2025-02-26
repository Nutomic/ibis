use crate::{components::suspense_error::SuspenseError, utils::resources::site};
use ibis_api_client::{CLIENT, errors::FrontendResultExt, user::UpdateUserParams};
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

    // TODO: It would make sense to use a table for the labels and inputs, but for some reason
    //       that completely breaks reactivity.
    view! {
        <Title text="Edit Profile" />
        <SuspenseError result=site()>
            {Suspend::new(async move {
                site()
                    .await
                    .ok()
                    .and_then(|site| site.my_profile)
                    .map(|my_profile| {
                        let (display_name, set_display_name) = signal(
                            my_profile.person.display_name.clone().unwrap_or_default(),
                        );
                        let (bio, set_bio) = signal(
                            my_profile.person.bio.clone().unwrap_or_default(),
                        );
                        view! {
                            <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                                Edit Profile
                            </h1>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40">Displayname</label>
                                <input
                                    type="text"
                                    id="displayname"
                                    class="w-80 input input-secondary input-bordered"
                                    bind:value=(display_name, set_display_name)
                                />
                            </div>
                            <div class="flex flex-row mb-2">
                                <label class="block w-40" for="bio">
                                    "Bio (Markdown supported)"
                                </label>
                                <textarea
                                    id="bio"
                                    class="w-80 text-base textarea textarea-secondary"
                                    bind:value=(bio, set_bio)
                                >
                                    bio.get()
                                </textarea>
                            </div>
                            <button
                                class="btn btn-primary"
                                on:click=move |_| {
                                    let form = UpdateUserParams {
                                        display_name: Some(display_name.get()),
                                        bio: Some(bio.get()),
                                    };
                                    submit_action.dispatch(form);
                                }
                            >
                                Submit
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
