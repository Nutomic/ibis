use leptos::*;
use leptos_router::*;

#[component]
pub fn Article() -> impl IntoView {
    view! {
        <nav class="inner">
            <li>
                <A href="read">"Read"</A>
            </li>
            <li>
                <A href="edit">"Edit"</A>
            </li>
        </nav>
    }
}
