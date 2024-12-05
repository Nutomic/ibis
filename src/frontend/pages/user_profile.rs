use crate::{
    common::GetUserForm,
    frontend::{api::CLIENT, components::edit_list::EditList, user_title},
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let name = move || params.get().get("name").clone().unwrap_or_default();
    let (error, set_error) = signal(None::<String>);
    let user_profile = Resource::new(name, move |mut name| async move {
        set_error.set(None);
        let mut domain = None;
        if let Some((title_, domain_)) = name.clone().split_once('@') {
            name = title_.to_string();
            domain = Some(domain_.to_string());
        }
        let params = GetUserForm { name, domain };
        CLIENT.get_user(params).await.unwrap()
    });

    let edits = Resource::new(
        move || user_profile.get(),
        move |_| async move {
            CLIENT
                .get_person_edits(user_profile.await.id)
                .await
                .unwrap_or_default()
        },
    );

    view! {
        {move || {
            error
                .get()
                .map(|err| {
                    view! { <p style="color:red;">{err}</p> }
                })
        }}

        <Suspense fallback=|| {
            view! { "Loading..." }
        }>
            {move || Suspend::new(async move {
                let edits = edits.await;
                let person = user_profile.await;
                view! {
                    <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                        {user_title(&person)}
                    </h1>

                    <h2 class="font-serif text-xl font-bold">Edits</h2>
                    <EditList edits=edits for_article=false />
                }
            })}

        </Suspense>
    }
}
