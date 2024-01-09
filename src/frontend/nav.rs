use leptos::{component, view, IntoView};
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <nav class="inner">
            <li>
                <A href="/">"Main Page"</A>
            </li>
            <li>
                <A href="/login">"Login"</A>
            </li>
            <li>
                <A href="/register">"Register"</A>
            </li>
        </nav>
    }
}
