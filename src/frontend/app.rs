use crate::common::LocalUserView;
use crate::frontend::api::ApiClient;
use crate::frontend::components::nav::Nav;
use crate::frontend::pages::article::create::CreateArticle;
use crate::frontend::pages::article::edit::EditArticle;
use crate::frontend::pages::article::history::ArticleHistory;
use crate::frontend::pages::article::list::ListArticles;
use crate::frontend::pages::article::read::ReadArticle;
use crate::frontend::pages::diff::EditDiff;
use crate::frontend::pages::instance_details::InstanceDetails;
use crate::frontend::pages::login::Login;
use crate::frontend::pages::register::Register;
use crate::frontend::pages::search::Search;
use crate::frontend::pages::user_profile::UserProfile;
use leptos::{
    component, create_local_resource, create_rw_signal, expect_context, provide_context,
    use_context, view, IntoView, RwSignal, SignalGet, SignalGetUntracked, SignalUpdate,
};
use leptos_meta::provide_meta_context;
use leptos_meta::*;
use leptos_router::Route;
use leptos_router::Router;
use leptos_router::Routes;
use reqwest::Client;

// https://book.leptos.dev/15_global_state.html
#[derive(Clone)]
pub struct GlobalState {
    api_client: ApiClient,
    pub(crate) my_profile: Option<LocalUserView>,
}

impl GlobalState {
    pub fn api_client() -> ApiClient {
        use_context::<RwSignal<GlobalState>>()
            .expect("global state is provided")
            .get_untracked()
            .api_client
    }

    pub fn update_my_profile() {
        create_local_resource(
            move || (),
            |_| async move {
                let my_profile = GlobalState::api_client().my_profile().await.ok();
                expect_context::<RwSignal<GlobalState>>()
                    .update(|state| state.my_profile = my_profile.clone());
            },
        );
    }

    pub fn is_admin() -> fn() -> bool {
        move || {
            use_context::<RwSignal<GlobalState>>()
                .expect("global state is provided")
                .get()
                .my_profile
                .map(|p| p.local_user.admin)
                .unwrap_or(false)
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let backend_hostname;
    #[cfg(not(feature = "ssr"))]
    {
        backend_hostname = web_sys::window().unwrap().location().host().unwrap();
    }
    #[cfg(feature = "ssr")]
    {
        backend_hostname = crate::backend::config::IbisConfig::read().bind.to_string();
    }

    provide_meta_context();
    let backend_hostname = GlobalState {
        api_client: ApiClient::new(Client::new(), backend_hostname.clone()),
        my_profile: None,
    };
    // Load user profile in case we are already logged in
    GlobalState::update_my_profile();
    provide_context(create_rw_signal(backend_hostname));

    view! {
        <>
            <Stylesheet id="simple" href="/assets/simple.css"/>
            <Stylesheet id="ibis" href="/assets/ibis.css"/>
            <Router>
                <Nav />
                <main>
                    <Routes>
                        <Route path="/" view=ReadArticle/>
                        <Route path="/article/:title" view=ReadArticle/>
                        <Route path="/article/:title/edit" view=EditArticle/>
                        <Route path="/article/:title/history" view=ArticleHistory/>
                        <Route path="/article/:title/diff/:hash" view=EditDiff/>
                        <Route path="/article/create" view=CreateArticle/>
                        <Route path="/article/list" view=ListArticles/>
                        <Route path="/instance/:hostname" view=InstanceDetails/>
                        <Route path="/user/:name" view=UserProfile/>
                        <Route path="/login" view=Login/>
                        <Route path="/register" view=Register/>
                        <Route path="/search" view=Search/>
                    </Routes>
                </main>
            </Router>
        </>
    }
}
