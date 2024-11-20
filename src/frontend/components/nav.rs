use crate::frontend::{api::CLIENT, app::is_logged_in};
use leptos::{component, prelude::*, view, IntoView};
use leptos_router::components::A;

#[component]
pub fn Nav() -> impl IntoView {
    let notification_count = Resource::new(
        || (),
        move |_| async move { CLIENT.notifications_count().await.unwrap_or_default() },
    );

    view! {
        <nav>
            <Transition>
                <Show when=is_logged_in>
                    <li>
                        <A href="/article/create">"Create Article"</A>
                    </li>
                    <li>
                        <A href="/notifications">
                            "Notifications "
                            <span class="indicator-item indicator-end badge badge-neutral">
                                {move || notification_count.get()}
                            </span>
                        </A>
                    </li>
                </Show>
            </Transition>
        </nav>
    }
}
