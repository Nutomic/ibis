use ibis_frontend_components::{
    suspense_error::SuspenseError,
    utils::{formatting::user_link, resources::site},
};
use leptos::prelude::*;
use leptos_fluent::tr;
use leptos_meta::Title;

#[component]
pub fn About() -> impl IntoView {
    let site = site();
    view! {
        <Title text=move || tr!("about") />
        <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">{move || tr!("about")}</h1>
        <SuspenseError result=site>
            {move || Suspend::new(async move {
                let site = site.await;
                site.ok()
                    .map(|site| {
                        view! {
                            <div>"Topic: "{site.instance.topic}</div>
                            <div>"Administrated by: "{user_link(&site.admin)}</div>
                        }
                    })
            })}
        </SuspenseError>
    }
}
