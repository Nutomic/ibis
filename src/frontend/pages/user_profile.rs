use crate::{
    common::{DbPerson, GetUserForm},
    frontend::{api::CLIENT, user_title},
};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let name = move || params.get().get("name").clone().unwrap();
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
            {move || {
                user_profile
                    .get()
                    .map(|person: DbPerson| {
                        view! {
                            <h1 class="text-4xl font-bold font-serif my-6 grow flex-auto">
                                {user_title(&person)}
                            </h1>
                            <p>TODO: create actual user profile</p>
                        }
                    })
            }}

        </Suspense>
    }
}
