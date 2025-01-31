use crate::{
    common::user::GetUserParams,
    frontend::{
        api::CLIENT,
        components::{edit_list::EditList, suspense_error::SuspenseError},
        markdown::render_article_markdown,
        utils::formatting::user_title,
    },
};
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

#[component]
pub fn UserProfile() -> impl IntoView {
    let params = use_params_map();
    let name = move || params.get().get("name").clone().unwrap_or_default();
    let user_profile = Resource::new(name, move |mut name| async move {
        let mut domain = None;
        if let Some((title_, domain_)) = name.clone().split_once('@') {
            name = title_.to_string();
            domain = Some(domain_.to_string());
        }
        let params = GetUserParams { name, domain };
        CLIENT.get_user(params).await
    });

    view! {
        <SuspenseError result=user_profile>
            {move || Suspend::new(async move {
                let edits = Resource::new(
                    move || user_profile.get(),
                    move |_| async move { CLIENT.get_person_edits(user_profile.await?.id).await },
                );
                user_profile
                    .await
                    .map(|person| {
                        view! {
                            <Title text=user_title(&person) />
                            <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">
                                {user_title(&person)}
                            </h1>

                            <div
                                class="mb-2 max-w-full prose prose-slate"
                                inner_html=render_article_markdown(&person.bio.unwrap_or_default())
                            ></div>

                            <SuspenseError result=user_profile>
                                {move || Suspend::new(async move {
                                    edits
                                        .await
                                        .map(|edits| {
                                            view! {
                                                <h2 class="font-serif text-xl font-bold">Edits</h2>
                                                <EditList edits=edits for_article=false />
                                            }
                                        })
                                })}
                            </SuspenseError>
                        }
                    })
            })}
        </SuspenseError>
    }
}
