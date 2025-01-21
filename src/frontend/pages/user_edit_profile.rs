use crate::{
    common::user::UpdateUserParams,
    frontend::{
        api::CLIENT,
        app::{site, DefaultResource},
    },
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn UserEditProfile() -> impl IntoView {
    let (saved, set_saved) = signal(false);
    let (submit_error, set_submit_error) = signal(None::<String>);

    let submit_action = Action::new(move |params: &UpdateUserParams| {
        let params = params.clone();
        async move {
            let result = CLIENT.update_user_profile(params).await;
            match result {
                Ok(_res) => {
                    site().refetch();
                    set_saved.set(true);
                    set_submit_error.set(None);
                }
                Err(err) => {
                    let msg = err.to_string();
                    log::warn!("Unable to update profile: {msg}");
                    set_submit_error.set(Some(msg));
                }
            }
        }
    });

    // TODO: It would make sense to use a table for the labels and inputs, but for some reason
    //       that completely breaks reactivity.
    view! {
        <Title text="Edit Profile" />
        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {
                let my_profile = site().with_default(|site| site.clone().my_profile.unwrap());
                let (display_name, set_display_name) = signal(
                    my_profile.person.display_name.clone().unwrap_or_default(),
                );
                let (bio, set_bio) = signal(my_profile.person.bio.clone().unwrap_or_default());
                view! {
                    <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">Edit Profile</h1>
                    {move || {
                        submit_error
                            .get()
                            .map(|err| {
                                view! { <p class="alert alert-error">{err}</p> }
                            })
                    }}
                    <div class="flex flex-row mb-2">
                        <label class="block w-40">Displayname</label>
                        <input
                            type="text"
                            id="displayname"
                            class="w-80 input input-secondary input-bordered"
                            prop:value=display_name
                            value=display_name
                            on:change=move |ev| {
                                let val = event_target_value(&ev);
                                set_display_name.set(val);
                            }
                        />
                    </div>
                    <div class="flex flex-row mb-2">
                        <label class="block w-40" for="bio">
                            "Bio (Markdown supported)"
                        </label>
                        <textarea
                            id="bio"
                            class="w-80 text-base textarea textarea-secondary"
                            prop:value=move || bio.get()
                            on:input:target=move |evt| {
                                let val = evt.target().value();
                                set_bio.set(val);
                            }
                        >
                            bio.get()
                        </textarea>
                    </div>
                    <button
                        class="btn btn-primary"
                        on:click=move |_| {
                            let form = UpdateUserParams {
                                person_id: my_profile.person.id,
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
            }

        </Suspense>
    }
}
