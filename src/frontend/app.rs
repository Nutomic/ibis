use crate::{
    common::SiteView,
    frontend::{api::CLIENT, components::nav::Nav, pages::notifications::Notifications},
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, *};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

pub fn site() -> Resource<SiteView> {
    use_context::<Resource<SiteView>>().unwrap()
}

pub fn is_logged_in() -> bool {
    site().with_default(|site| site.my_profile.is_some())
}
pub fn is_admin() -> bool {
    site().with_default(|site| {
        site.my_profile
            .as_ref()
            .map(|p| p.local_user.admin)
            .unwrap_or(false)
    })
}

// TODO: can probably get rid of this
pub trait DefaultResource<T> {
    fn with_default<O>(&self, f: impl FnOnce(&T) -> O) -> O;
}

impl<T: Default + Send + Sync> DefaultResource<T> for Resource<T> {
    fn with_default<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        self.with(|x| match x {
            Some(x) => f(x),
            None => f(&T::default()),
        })
    }
}
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

    // TODO: should Resource::new() but then things break
    let site_resource = Resource::new(|| (), |_| async move { CLIENT.site().await.unwrap() });
    provide_context(site_resource);

    view! {
        <Html attr:data-theme=darkmode.theme {..} class="h-full" />
        <Body {..} class="h-full max-sm:flex max-sm:flex-col" />
        <>
            <Stylesheet id="ibis" href="/pkg/ibis.css" />
            <Stylesheet id="katex" href="/katex.min.css" />
            <Router>
                <Nav />
                <main class="p-4 md:ml-64">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=path!("/") view=Notifications />
                    </Routes>
                </main>
            </Router>
        </>
    }
}
