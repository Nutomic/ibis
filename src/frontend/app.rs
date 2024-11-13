use crate::{
    common::{LocalUserView, SiteView},
    frontend::{
        api::ApiClient,
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
use leptos::{
*};
use leptos_meta::{provide_meta_context, *};
use leptos_router::{Route, Router, Routes};
use std::{thread::sleep, time::Duration};

// https://book.leptos.dev/15_global_state.html
#[derive(Clone)]
pub struct GlobalState {
    api_client: ApiClient,
    pub(crate) my_profile: Option<LocalUserView>,
}

impl GlobalState {
    pub fn api_client() -> ApiClient {
        let mut global_state = use_context::<RwSignal<GlobalState>>();
        // Wait for global state to be populated (only needed on instance_details for some reason)
        while global_state.is_none() {
            sleep(Duration::from_millis(10));
            global_state = use_context::<RwSignal<GlobalState>>();
        }
        global_state
            .expect("global state is provided")
            .get_untracked()
            .api_client
    }
}

pub fn site() -> Resource<(), SiteView>{
    use_context::<Resource<(), SiteView>>().unwrap()
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let global_state = GlobalState {
        api_client: ApiClient::get().clone(),
        my_profile: None,
    };
    let site = create_resource(|| (), |_| async move {
        ApiClient::get().site().await.unwrap()
    });
    provide_context(site);
    provide_context(create_rw_signal(global_state));

    let darkmode = DarkMode::init();
    provide_context(darkmode.clone());

    view! {
        <Html attr:data-theme=darkmode.theme class="h-full" />
        <Body class="min-h-full flex max-sm:flex-col md:divide-x divide-slate-400 divide-solid" />
        <>
            <Stylesheet id="ibis" href="/pkg/ibis.css" />
            <Stylesheet id="katex" href="/katex.min.css" />
            <Router>
                <Nav />
                <main class="p-4 grow">
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
