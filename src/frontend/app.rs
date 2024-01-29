use crate::common::LocalUserView;
use crate::frontend::api::ApiClient;
use crate::frontend::components::nav::Nav;
use crate::frontend::pages::article::edit::EditArticle;
use crate::frontend::pages::article::history::ArticleHistory;
use crate::frontend::pages::article::read::ReadArticle;
use crate::frontend::pages::diff::EditDiff;
use crate::frontend::pages::login::Login;
use crate::frontend::pages::register::Register;
use crate::frontend::pages::Page;
use leptos::{
    component, create_local_resource, create_rw_signal, expect_context, provide_context,
    use_context, view, IntoView, RwSignal, SignalGetUntracked, SignalUpdate,
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

    pub fn update_my_profile(&self) {
        create_local_resource(
            move || (),
            |_| async move {
                let my_profile = GlobalState::api_client().my_profile().await.ok();
                expect_context::<RwSignal<GlobalState>>()
                    .update(|state| state.my_profile = my_profile.clone());
            },
        );
    }
}

#[component]
pub fn App() -> impl IntoView {
    let backend_hostname = "127.0.0.1:8080".to_string();

    provide_meta_context();
    let backend_hostname = GlobalState {
        api_client: ApiClient::new(Client::new(), backend_hostname.clone()),
        my_profile: None,
    };
    // Load user profile in case we are already logged in
    backend_hostname.update_my_profile();
    provide_context(create_rw_signal(backend_hostname));

    view! {
        <>
            <Stylesheet id="simple" href="/assets/simple.css"/>
            <Stylesheet id="ibis" href="/assets/ibis.css"/>
            <Router>
                <Nav />
                <main>
                    <Routes>
                        <Route path={Page::Home.path()} view=ReadArticle/>
                        <Route path="/article/:title" view=ReadArticle/>
                        <Route path="/article/:title/edit" view=EditArticle/>
                        <Route path="/article/:title/history" view=ArticleHistory/>
                        <Route path="/article/:title/diff/:hash" view=EditDiff/>
                        <Route path={Page::Login.path()} view=Login/>
                        <Route path={Page::Register.path()} view=Register/>
                    </Routes>
                </main>
            </Router>
        </>
    }
}
