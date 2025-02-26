use crate::{
    components::suspense_error::SuspenseError,
    utils::formatting::{
        article_link,
        article_path,
        article_title,
        comment_path,
        edit_path,
        time_ago,
        user_link,
    },
};
use chrono::{DateTime, Utc};
use ibis_api_client::{
    CLIENT,
    errors::{FrontendError, FrontendResultExt},
};
use ibis_database::common::{
    article::{Article, Conflict, Edit},
    comment::Comment,
    newtypes::ArticleNotifId,
    notifications::ApiNotification,
    user::Person,
};
use leptos::{either::EitherOf4, prelude::*};
use leptos_meta::Title;
use phosphor_leptos::{CHECK, Icon, LINK};

type NotificationsResource = Resource<Result<Vec<ApiNotification>, FrontendError>>;

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
                                    use ApiNotification::*;
                                    match notif {
                                        EditConflict(c, a) => {
                                            EitherOf4::A(edit_conflict_view(c, a, notifications))
                                        }
                                        ArticleApprovalRequired(a) => {
                                            EitherOf4::B(article_approval_view(a, notifications))
                                        }
                                        Comment(id, c, p, a) => {
                                            EitherOf4::C(comment_view(*id, c, p, a, notifications))
                                        }
                                        Edit(id, e, p, a) => {
                                            EitherOf4::D(edit_view(*id, e, p, a, notifications))
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

fn edit_conflict_view(
    c: &Conflict,
    a: &Article,
    notifications: NotificationsResource,
) -> impl IntoView {
    let link = format!("{}/edit?conflict_id={}", article_path(a), c.id.0,);
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
                {format!("Conflict: {} - {}", article_title(a), c.summary)}
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

fn article_approval_view(a: &Article, notifications: NotificationsResource) -> impl IntoView {
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

fn comment_view(
    id: ArticleNotifId,
    comment: &Comment,
    creator: &Person,
    article: &Article,
    notifications: NotificationsResource,
) -> impl IntoView {
    let click_mark_as_read = Action::new(move |_: &()| async move {
        CLIENT
            .article_notif_mark_as_read(id)
            .await
            .error_popup(|_| notifications.refetch());
    });
    view! {
        <li class="py-2">
            <CardTitle article=article.clone() creator=creator.clone() time=comment.published />
            <div>{comment.content.clone()}</div>
            <CardActions
                href=comment_path(comment, article)
                action=move || {
                    click_mark_as_read.dispatch(());
                }
            />
        </li>
    }
}

fn edit_view(
    id: ArticleNotifId,
    edit: &Edit,
    creator: &Person,
    article: &Article,
    notifications: NotificationsResource,
) -> impl IntoView {
    let click_mark_as_read = Action::new(move |_: &()| async move {
        CLIENT
            .article_notif_mark_as_read(id)
            .await
            .error_popup(|_| notifications.refetch());
    });
    let mark_as_read_action = move || {
        click_mark_as_read.dispatch(());
    };
    view! {
        <li class="py-2">
            <CardTitle article=article.clone() creator=creator.clone() time=edit.published />
            <div>{edit.summary.clone()}</div>
            <CardActions href=edit_path(edit, article) action=mark_as_read_action />
        </li>
    }
}

#[component]
fn CardTitle(article: Article, creator: Person, time: DateTime<Utc>) -> impl IntoView {
    view! {
        <div class="flex text-s">
            <span class="grow">{user_link(&creator)}" - "{article_link(&article)}</span>
            {time_ago(time)}
        </div>
    }
}

#[component]
fn CardActions<F>(href: String, action: F) -> impl IntoView
where
    F: Fn() + 'static,
{
    view! {
        <div class="mt-2 card-actions">
            <a class="btn btn-sm btn-outline" href=href title="View">
                <Icon icon=LINK />
            </a>
            <button class="btn btn-sm btn-outline" on:click=move |_| action() title="Mark as read">
                <Icon icon=CHECK />
            </button>
        </div>
    }
}
