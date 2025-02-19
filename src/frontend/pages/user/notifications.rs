use crate::{
    common::{
        article::{ApiConflict, DbArticle},
        comment::CommentViewWithArticle,
        notifications::{ArticleNotificationKind, ArticleNotificationView, Notification},
    },
    frontend::{
        api::CLIENT,
        components::suspense_error::SuspenseError,
        utils::{
            errors::{FrontendError, FrontendResultExt},
            formatting::{
                article_link, article_path, article_title, comment_path, time_ago, user_link,
            },
        },
    },
};
use leptos::{
    either::{Either, EitherOf4},
    prelude::*,
};
use leptos_meta::Title;
use phosphor_leptos::{Icon, CHECK, LINK};

type NotificationsResource = Resource<Result<Vec<Notification>, FrontendError>>;

#[component]
pub fn Notifications() -> impl IntoView {
    let notifications = Resource::new(
        move || {},
        |_| async move { CLIENT.notifications_list().await },
    );

    view! {
        <Title text="Notifications" />
        <h1 class="flex-auto my-6 font-serif text-4xl font-bold grow">Notifications</h1>
        <SuspenseError result=notifications>
            <ul class="divide-y divide-solid">
                {move || Suspend::new(async move {
                    notifications
                        .await
                        .map(|n| {
                            n.into_iter()
                                .map(|ref notif| {
                                    use Notification::*;
                                    match notif {
                                        EditConflict(c) => {
                                            EitherOf4::A(edit_conflict_view(c, notifications))
                                        }
                                        ArticleApprovalRequired(a) => {
                                            EitherOf4::B(article_approval_view(a, notifications))
                                        }
                                        Reply(c) => EitherOf4::C(reply_view(c, notifications)),
                                        ArticleNotification(n) => {
                                            EitherOf4::D(article_notification_view(n, notifications))
                                        }
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                })}

            </ul>
        </SuspenseError>
    }
}

fn edit_conflict_view(c: &ApiConflict, notifications: NotificationsResource) -> impl IntoView {
    let link = format!("{}/edit?conflict_id={}", article_path(&c.article), c.id.0,);
    let id = c.id;
    let click_dismiss = Action::new(move |_: &()| async move {
        CLIENT
            .delete_conflict(id)
            .await
            .error_popup(|_| notifications.refetch());
    });
    view! {
        <li class="py-2">
            <a class="text-lg link" href=link>
                {format!("Conflict: {} - {}", article_title(&c.article), c.summary)}
            </a>
            <div class="mt-2 card-actions">
                <button
                    class="btn btn-sm btn-outline"
                    on:click=move |_| {
                        click_dismiss.dispatch(());
                    }
                >
                    Dismiss
                </button>
            </div>
        </li>
    }
}

fn article_approval_view(a: &DbArticle, notifications: NotificationsResource) -> impl IntoView {
    let id = a.id;
    let click_approve = Action::new(move |_: &()| async move {
        CLIENT
            .approve_article(id, true)
            .await
            .error_popup(|_| notifications.refetch());
    });
    let click_reject = Action::new(move |_: &()| async move {
        CLIENT
            .approve_article(id, false)
            .await
            .error_popup(|_| notifications.refetch());
    });
    view! {
        <li class="py-2">
            <a class="text-lg link" href=article_path(a)>
                {format!("Approval required: {}", a.title)}
            </a>
            <div class="mt-2 card-actions">
                <button
                    class="btn btn-sm btn-outline"
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
}

fn reply_view(c: &CommentViewWithArticle, notifications: NotificationsResource) -> impl IntoView {
    let id = c.comment.id;
    let click_mark_as_read = Action::new(move |_: &()| async move {
        CLIENT
            .mark_comment_as_read(id)
            .await
            .error_popup(|_| notifications.refetch());
    });
    view! {
        <li class="py-2">
            <div class="flex text-s">
                <span class="grow">{user_link(&c.creator)}" - "{article_link(&c.article)}</span>
                {time_ago(c.comment.published)}
            </div>
            <div>{c.comment.content.clone()}</div>
            <div class="mt-2 card-actions">
                <ButtonLink href=comment_path(&c.comment, &c.article) />
                <ButtonMarkAsRead action=move || {
                    click_mark_as_read.dispatch(());
                } />
            </div>
        </li>
    }
}

fn article_notification_view(
    n: &ArticleNotificationView,
    notifications: NotificationsResource,
) -> impl IntoView {
    use ArticleNotificationKind::*;
    let article_path = article_path(&n.article);
    let article_title = n.article.title.clone();
    let id = n.id;
    let click_mark_as_read = Action::new(move |_: &()| async move {
        CLIENT
            .article_notif_mark_as_read(id)
            .await
            .error_popup(|_| notifications.refetch());
    });
    let mark_as_read_action = move || {
        click_mark_as_read.dispatch(());
    };
    match n.kind {
        Comment => Either::Left(view! {
            <li class="py-2">
                <div class="flex text-s">
                    <span class="grow">"New comment on article " {article_title}</span>
                    {time_ago(n.published)}
                </div>
                <div class="mt-2 card-actions">
                    <ButtonLink href=format!("{article_path}/discussion") />
                    <ButtonMarkAsRead action=mark_as_read_action />
                </div>
            </li>
        }),
        Edit => Either::Right(view! {
            <li class="py-2">
                <div class="flex text-s">
                    <span class="grow">"New edit on article " {article_title}</span>
                    {time_ago(n.published)}
                </div>
                <div class="mt-2 card-actions">
                    <ButtonLink href=format!("{article_path}/history") />
                    <ButtonMarkAsRead action=mark_as_read_action />
                </div>
            </li>
        }),
    }
}

#[component]
fn ButtonLink(href: String) -> impl IntoView {
    view! {
        <a class="btn btn-sm btn-outline" href=href title="View">
            <Icon icon=LINK />
        </a>
    }
}

#[component]
fn ButtonMarkAsRead<F>(action: F) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <button class="btn btn-sm btn-outline" on:click=move |_| action() title="Mark as read">
            <Icon icon=CHECK />
        </button>
    }
}
