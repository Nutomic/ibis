use crate::{
    common::Notification,
    frontend::{app::GlobalState, article_link, article_title},
};
use leptos::*;

#[component]
pub fn Notifications() -> impl IntoView {
    let notifications = create_local_resource(
        move || {},
        |_| async move {
            GlobalState::api_client()
                .notifications_list()
                .await
                .unwrap()
        },
    );

    view! {
        <h1 class="text-4xl font-bold font-serif my-6 grow flex-auto">Notifications</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <ul>
                {move || {
                    notifications
                        .get()
                        .map(|n| {
                            n.into_iter()
                                .map(|n| {
                                    use Notification::*;
                                    let (link, title) = match n {
                                        EditConflict(c) => (format!(
                                            "{}/edit/{}",
                                            article_link(&c.article),
                                            c.id.0)
                                        , format!("Conflict: {} - {}", article_title(&c.article), c.summary)),
                                        ArticleApprovalRequired(a) => (article_link(&a), format!("Approval required: {}", a.title)),

                                    };
                                    // TODO: need buttons to approve/reject new article, also makes sense to discard edit conflict
                                    view! {
                                        <li>
                                            <a class="link text-lg" href=link>
                                                {title}
                                            </a>
                                        </li>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                }}

            </ul>
        </Suspense>
    }
}
