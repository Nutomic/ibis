use crate::{
    common::instance::SiteView,
    frontend::{
        api::CLIENT,
        components::{nav::Nav, protected_route::IbisProtectedRoute},
        dark_mode::DarkMode,
        instance_title,
        pages::{
            article::{
                actions::ArticleActions,
                create::CreateArticle,
                discussion::ArticleDiscussion,
                edit::EditArticle,
                history::ArticleHistory,
                list::ListArticles,
                read::ReadArticle,
            },
            diff::EditDiff,
            instance::{details::InstanceDetails, list::ListInstances, settings::InstanceSettings},
            login::Login,
            notifications::Notifications,
            register::Register,
            search::Search,
            user_edit_profile::UserEditProfile,
            user_profile::UserProfile,
        },
    },
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
pub trait DefaultResource<T> {
    fn with_default<O>(&self, f: impl FnOnce(&T) -> O) -> O;
    fn get_default(&self) -> T;
}

impl<T: Default + Send + Sync + Clone> DefaultResource<T> for Resource<T> {
    fn with_default<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        self.with(|x| match x {
            Some(x) => f(x),
            None => f(&T::default()),
        })
    }
    fn get_default(&self) -> T {
        match self.get() {
            Some(x) => x.clone(),
            None => T::default(),
        }
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

    let site_resource = Resource::new(|| (), |_| async move { CLIENT.site().await.unwrap() });
    provide_context(site_resource);

    let darkmode = DarkMode::init();
    provide_context(darkmode.clone());

    let instance = Resource::new(
        || (),
        |_| async move { CLIENT.get_local_instance().await.unwrap() },
    );
    view! {
        <Html attr:data-theme=darkmode.theme {..} class="h-full" />
        <Body {..} class="h-full max-sm:flex max-sm:flex-col" />
        <>
            <Stylesheet id="ibis" href="/pkg/ibis.css" />
            <Stylesheet id="katex" href="/katex.min.css" />
            <Router>
                <Suspense>
                    {move || {
                        instance
                            .get()
                            .map(|i| {
                                let formatter = move |text| {
                                    format!("{text} — {}", instance_title(&i.instance))
                                };
                                view! { <Title formatter /> }
                            })
                    }}
                </Suspense>
                <Nav />
                <main class="p-4 md:ml-64">
                    <Routes fallback=|| "Page not found.".into_view()>
                        <Route path=path!("/") view=ReadArticle />
                        <Route path=path!("/article/:title") view=ReadArticle />
                        <Route path=path!("/article/:title/discussion") view=ArticleDiscussion />
                        <Route path=path!("/article/:title/history") view=ArticleHistory />
                        <IbisProtectedRoute
                            path=path!("/article/:title/edit/:conflict_id?")
                            view=EditArticle
                        />
                        <IbisProtectedRoute
                            path=path!("/article/:title/actions")
                            view=ArticleActions
                        />
                        <Route path=path!("/article/:title/diff/:hash") view=EditDiff />
                        <IbisProtectedRoute path=path!("/create-article") view=CreateArticle />
                        <Route path=path!("/articles") view=ListArticles />
                        <Route path=path!("/instances") view=ListInstances />
                        <Route path=path!("/instance/:hostname") view=InstanceDetails />
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
