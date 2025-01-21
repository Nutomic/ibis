use crate::{
    common::Notification,
    frontend::{api::CLIENT, article_path, article_title},
};
use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn Notifications() -> impl IntoView {
    let notifications = Resource::new(
        move || {},
        |_| async move { CLIENT.notifications_list().await.unwrap_or_default() },
    );

    view! {
        <Title text="Notifications" />
        <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">Notifications</h1>
        <Suspense fallback=|| view! { "Loading..." }>
            <ul class="divide-y divide-solid">
                {move || {
                    notifications
                        .get()
                        .map(|n| {
                            n.into_iter()
                                .map(|ref notif| {
                                    use Notification::*;
                                    let (my_style, link, title) = match notif {
                                        EditConflict(c) => {
                                            (
                                                "visibility: hidden",
                                                format!("{}/edit/{}", article_path(&c.article), c.id.0),
                                                format!(
                                                    "Conflict: {} - {}",
                                                    article_title(&c.article),
                                                    c.summary,
                                                ),
                                            )
                                        }
                                        ArticleApprovalRequired(a) => {
                                            (
                                                "",
                                                article_path(a),
                                                format!("Approval required: {}", a.title),
                                            )
                                        }
                                    };
                                    let notif_ = notif.clone();
                                    let click_approve = Action::new(move |_: &()| {
                                        let notif_ = notif_.clone();
                                        async move {
                                            if let ArticleApprovalRequired(a) = notif_ {
                                                CLIENT.approve_article(a.id, true).await.unwrap();
                                            }
                                            notifications.refetch();
                                        }
                                    });
                                    let notif_ = notif.clone();
                                    let click_reject = Action::new(move |_: &()| {
                                        let notif_ = notif_.clone();
                                        async move {
                                            match notif_ {
                                                EditConflict(c) => {
                                                    CLIENT.delete_conflict(c.id).await.unwrap();
                                                }
                                                ArticleApprovalRequired(a) => {
                                                    CLIENT.approve_article(a.id, false).await.unwrap();
                                                }
                                            }
                                            notifications.refetch();
                                        }
                                    });
                                    view! {
                                        <li class="py-2">
                                            <a class="text-lg link" href=link>
                                                {title}
                                            </a>
                                            <div class="mt-2 card-actions">
                                                <button
                                                    class="btn btn-sm btn-outline"
                                                    style=my_style
                                                    on:click=move |_| {
                                                        click_approve.dispatch(());
                                                    }
                                                >
                                                    Approve
                                                </button>
                                                <button
                                                    class="btn btn-sm btn-outline"
                                                    on:click=move |_| {
                                                        click_reject.dispatch(());
                                                    }
                                                >
                                                    Reject
                                                </button>
                                            </div>
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
