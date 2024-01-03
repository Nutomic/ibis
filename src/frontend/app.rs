use crate::frontend::article::Article;
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
            <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
            <Stylesheet id="leptos" href="/style.css"/>
            <Meta name="description" content="Leptos implementation of a HackerNews demo."/>
            <Router>
                <Nav />
                <main>
                    <Routes>
                        <Route path="/" view=Article/>
                    </Routes>
                </main>
            </Router>

        </>
    }
}
