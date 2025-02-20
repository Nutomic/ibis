use crate::frontend::{
    api::CLIENT,
    components::{nav::Nav, protected_route::IbisProtectedRoute},
    pages::{
        article::{
            actions::ArticleActions,
            create::CreateArticle,
            diff::EditDiff,
            discussion::ArticleDiscussion,
            edit::EditArticle,
            history::ArticleHistory,
            read::ReadArticle,
        },
        instance::{
            about::About,
            details::InstanceDetails,
            explore::Explore,
            search::Search,
            settings::InstanceSettings,
        },
        user::{
            edit_profile::UserEditProfile,
            login::Login,
            notifications::Notifications,
            profile::UserProfile,
            register::Register,
        },
    },
    utils::{dark_mode::DarkMode, errors::ErrorPopup, formatting::instance_title},
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, *};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

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

    let site_resource = Resource::new(|| (), |_| async move { CLIENT.site().await });
    provide_context(site_resource);

    let instance = Resource::new(|| (), |_| async move { CLIENT.get_local_instance().await });

    let darkmode = DarkMode::init();
    provide_context(darkmode.clone());

    ErrorPopup::init();

    view! {
        <Html attr:data-theme=darkmode.theme {..} class="h-full" />
        <Body {..} class="h-full max-sm:flex max-sm:flex-col" />
        <>
            <Stylesheet id="ibis" href="/pkg/ibis.css" />
            <Stylesheet id="katex" href="/katex.min.css" />
            <Router>
                <Nav />
                <main class="p-4 md:ml-64">
                    <Suspense>
                        {move || Suspend::new(async move {
                            instance
                                .await
                                .map(|i| {
                                    let formatter = move |text| {
                                        format!("{text} — {}", instance_title(&i.instance))
                                    };
                                    view! { <Title formatter /> }
                                })
                        })}
                    </Suspense>
                    <Show when=move || ErrorPopup::get().is_some()>
                        <div class="toast">
                            <div class="alert alert-error">
                                <span>{ErrorPopup::get()}</span>
                            </div>
                        </div>
                    </Show>
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=path!("/") view=ReadArticle />
                        <Route path=path!("/article/:title") view=ReadArticle />
                        <Route path=path!("/article/:title/discussion") view=ArticleDiscussion />
                        <Route path=path!("/article/:title/history") view=ArticleHistory />
                        <IbisProtectedRoute path=path!("/article/:title/edit") view=EditArticle />
                        <IbisProtectedRoute
                            path=path!("/article/:title/actions")
                            view=ArticleActions
                        />
                        <Route path=path!("/article/:title/diff/:hash") view=EditDiff />
                        <IbisProtectedRoute path=path!("/create-article") view=CreateArticle />
                        <Route path=path!("/explore") view=Explore />
                        <Route path=path!("/instance/:hostname") view=InstanceDetails />
                        <Route path=path!("/about") view=About />
                        <Route path=path!("/user/:name") view=UserProfile />
                        <Route path=path!("/login") view=Login />
                        <Route path=path!("/register") view=Register />
                        <Route path=path!("/search") view=Search />
                        <IbisProtectedRoute path=path!("/edit_profile") view=UserEditProfile />
                        <IbisProtectedRoute path=path!("/notifications") view=Notifications />
                        <IbisProtectedRoute path=path!("/settings") view=InstanceSettings />
                    </Routes>
                </main>
            </Router>
        </>
    }
}
