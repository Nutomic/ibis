use leptos::{component, view, IntoView};
use leptos_router::*;

#[component]
pub fn Nav() -> impl IntoView {
    view! {
        <div>
            <nav class="inner">
                <li>
                    <A href="/">
                        <strong>"Main Page"</strong>
                    </A>
                </li>
                <li>
                <A href="/latest">
                    <strong>"Latest changes"</strong>
                </A>
                </li>
            </nav>
        </div>
    }
}
