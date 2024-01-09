use crate::frontend::article::Article;
use crate::frontend::login::Login;
use crate::frontend::nav::Nav;
use leptos::{component, view, IntoView};
use leptos_meta::provide_meta_context;
use leptos_meta::*;
use leptos_router::Route;
use leptos_router::Router;
use leptos_router::Routes;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <>
            <Stylesheet id="simple" href="/assets/simple.css"/>
            <Stylesheet id="ibis" href="/assets/ibis.css"/>
            <Router>
                <Nav />
                <main>
                    <Routes>
                        <Route path="/" view=Article/>
                        <Route path="/login" view=Login/>
                    </Routes>
                </main>
            </Router>

        </>
    }
}
