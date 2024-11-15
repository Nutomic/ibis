use crate::{
    common::SiteView,
    frontend::{
        api::CLIENT,
        components::nav::Nav,
        dark_mode::DarkMode,
        pages::{
            article::{
                actions::ArticleActions,
                create::CreateArticle,
                edit::EditArticle,
                history::ArticleHistory,
                list::ListArticles,
                read::ReadArticle,
            },
            diff::EditDiff,
            instance::{details::InstanceDetails, list::ListInstances},
            login::Login,
            notifications::Notifications,
            register::Register,
            search::Search,
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

#[component]
pub fn App() -> impl IntoView {
    // TODO: should Resource::new() but then things break
    let site_resource = LocalResource::new(|| async move { CLIENT.site().await.unwrap() });
    provide_context(site_resource);
    provide_meta_context();

    let darkmode = DarkMode::init();
    provide_context(darkmode.clone());

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
                        <Route path=path!("/") view=ReadArticle />
                        <Route path=path!("/article/:title") view=ReadArticle />
                        <Route path=path!("/article/:title/history") view=ArticleHistory />
                        <Route path=path!("/article/:title/edit/:conflict_id?") view=EditArticle />
                        <Route path=path!("/article/:title/actions") view=ArticleActions />
                        <Route path=path!("/article/:title/diff/:hash") view=EditDiff />
                        // TODO: use protected route, otherwise user can view
                        // /article/create without login
                        // https://github.com/leptos-rs/leptos/blob/leptos_0.7/examples/router/src/lib.rs#L51
                        <Route path=path!("/article/create") view=CreateArticle />
                        <Route path=path!("/article/list") view=ListArticles />
                        <Route path=path!("/instance/:hostname") view=InstanceDetails />
                        <Route path=path!("/instance/list") view=ListInstances />
                        <Route path=path!("/user/:name") view=UserProfile />
                        <Route path=path!("/login") view=Login />
                        <Route path=path!("/register") view=Register />
                        <Route path=path!("/search") view=Search />
                        <Route path=path!("/notifications") view=Notifications />
                    </Routes>
                </main>
            </Router>
        </>
    }
}
