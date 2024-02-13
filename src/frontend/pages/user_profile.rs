use crate::common::{DbPerson, GetUserData};
use crate::frontend::app::GlobalState;
use crate::frontend::extract_domain;
use leptos::*;
use leptos_router::use_params_map;

#[component]
pub fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let name = move || params.get().get("name").cloned().unwrap();
    let (error, set_error) = create_signal(None::<String>);
    let user_profile = create_resource(name, move |mut name| async move {
        set_error.set(None);
        let mut domain = None;
        if let Some((title_, domain_)) = name.clone().split_once('@') {
            name = title_.to_string();
            domain = Some(domain_.to_string());
        }
        let params = GetUserData { name, domain };
        GlobalState::api_client().get_user(params).await.unwrap()
    });

    view! {
        {move || {
            error
                .get()
                .map(|err| {
                    view! { <p style="color:red;">{err}</p> }
                })
        }}
        <Suspense fallback=|| view! {  "Loading..." }> {
            move || user_profile.get().map(|person: DbPerson| {
                let name =
                if person.local {
                    person.username
                } else {
                    format!("{}@{}", person.username, extract_domain(&person.ap_id))
                };
                view! {
                    <h1>{name}</h1>
                }
            })
        }</Suspense>
    }
}