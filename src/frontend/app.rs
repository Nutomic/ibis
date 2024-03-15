use crate::{
    common::LocalUserView,
    frontend::{
        api::ApiClient,
        components::nav::Nav,
        pages::{
            article::{
                actions::ArticleActions,
                create::CreateArticle,
                edit::EditArticle,
                history::ArticleHistory,
                list::ListArticles,
                read::ReadArticle,
            },
            conflicts::Conflicts,
            diff::EditDiff,
            instance_details::InstanceDetails,
            login::Login,
            register::Register,
            search::Search,
            user_profile::UserProfile,
        },
    },
};
use leptos::{
    component,
    create_local_resource,
    create_rw_signal,
    expect_context,
    provide_context,
    use_context,
    view,
    IntoView,
    RwSignal,
    SignalGet,
    SignalGetUntracked,
    SignalUpdate,
};
use leptos_meta::{provide_meta_context, *};
use leptos_router::{Route, Router, Routes};
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
    provide_meta_context();
    let global_state = GlobalState {
        api_client: ApiClient::new(Client::new(), None),
        my_profile: None,
    };
    // Load user profile in case we are already logged in
    GlobalState::update_my_profile();
    provide_context(create_rw_signal(global_state));

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
                        <Route path="/article/:title/history" view=ArticleHistory/>
                        <Route path="/article/:title/edit/:conflict_id?" view=EditArticle/>
                        <Route path="/article/:title/actions" view=ArticleActions/>
                        <Route path="/article/:title/diff/:hash" view=EditDiff/>
                        <Route path="/article/create" view=CreateArticle/>
                        <Route path="/article/list" view=ListArticles/>
                        <Route path="/instance/:hostname" view=InstanceDetails/>
                        <Route path="/user/:name" view=UserProfile/>
                        <Route path="/login" view=Login/>
                        <Route path="/register" view=Register/>
                        <Route path="/search" view=Search/>
                        <Route path="/conflicts" view=Conflicts/>
                    </Routes>
                </main>
            </Router>
        </>
    }
}
