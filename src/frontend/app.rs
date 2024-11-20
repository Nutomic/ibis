use crate::common::SiteView;
use crate::frontend::api::CLIENT;
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, *};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let site_resource = Resource::new(|| (), |_| async move { CLIENT.site().await.unwrap() });
    provide_context(site_resource);

    view! {
        <Html />
        <Body />
        <>
            <Transition>
                <Show when=move || {
                    use_context::<Resource<SiteView>>()
                        .unwrap()
                        .get_untracked()
                        .unwrap_or_default()
                        .my_profile
                        .is_some()
                }>test</Show>
            </Transition>
        </>
    }
}
