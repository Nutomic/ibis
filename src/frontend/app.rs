use crate::frontend::components::nav::Nav;
use crate::frontend::pages::article::Article;
use crate::frontend::pages::login::Login;
use crate::frontend::pages::register::Register;
use crate::frontend::pages::Page;
use leptos::{component, provide_context, use_context, view, IntoView};
use leptos_meta::provide_meta_context;
use leptos_meta::*;
use leptos_router::Route;
use leptos_router::Router;
use leptos_router::Routes;

// TODO: change to GlobalState and also store auth token here
//       https://book.leptos.dev/15_global_state.html
#[derive(Clone)]
pub struct BackendHostname(String);

impl BackendHostname {
    pub fn read() -> String {
        use_context::<BackendHostname>()
            .expect("backend hostname is provided")
            .0
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let backend_hostname = BackendHostname("localhost:8080".to_string());
    provide_context(backend_hostname);
    view! {
        <>
            <Stylesheet id="simple" href="/assets/simple.css"/>
            <Stylesheet id="ibis" href="/assets/ibis.css"/>
            <Router>
                <Nav />
                <main>
                    <Routes>
                        <Route path={Page::Home.path()} view=Article/>
                        <Route path={Page::Login.path()} view=Login/>
                        <Route path={Page::Register.path()} view=Register/>
                    </Routes>
                </main>
            </Router>

        </>
    }
}
