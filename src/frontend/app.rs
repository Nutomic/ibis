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
use leptos::*;
use leptos_meta::{provide_meta_context, *};
use leptos_router::{Route, Router, Routes};

pub fn site() -> Resource<(), SiteView> {
    use_context::<Resource<(), SiteView>>().unwrap()
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
}

impl<T: Default> DefaultResource<T> for Resource<(), T> {
    fn with_default<O>(&self, f: impl FnOnce(&T) -> O) -> O {
        self.with(|x| match x {
            Some(x) => f(x),
            None => f(&T::default()),
        })
    }
}

#[component]
pub fn App() -> impl IntoView {
    // TODO: should create_resource() but then things break
    let site_resource = create_local_resource(
        move || (),
        |_| async move {
            let site = CLIENT.site().await.unwrap();
            site
        },
    );
    provide_context(site_resource);
    provide_meta_context();

    let darkmode = DarkMode::init();
    provide_context(darkmode.clone());

    view! {
        <Html attr:data-theme=darkmode.theme class="h-full" />
        <Body class="h-full max-sm:flex max-sm:flex-col" />
        <>
            <Stylesheet id="ibis" href="/pkg/ibis.css" />
            <Stylesheet id="katex" href="/katex.min.css" />
            <Router>
                <Nav />
                <main class="p-4 md:ml-64">
                    <Routes>
                        <Route path="/" view=ReadArticle />
                        <Route path="/article/:title" view=ReadArticle />
                        <Route path="/article/:title/history" view=ArticleHistory />
                        <Route path="/article/:title/edit/:conflict_id?" view=EditArticle />
                        <Route path="/article/:title/actions" view=ArticleActions />
                        <Route path="/article/:title/diff/:hash" view=EditDiff />
                        <Route path="/article/create" view=CreateArticle />
                        <Route path="/article/list" view=ListArticles />
                        <Route path="/instance/:hostname" view=InstanceDetails />
                        <Route path="/instance/list" view=ListInstances />
                        <Route path="/user/:name" view=UserProfile />
                        <Route path="/login" view=Login />
                        <Route path="/register" view=Register />
                        <Route path="/search" view=Search />
                        <Route path="/notifications" view=Notifications />
                    </Routes>
                </main>
            </Router>
        </>
    }
}
