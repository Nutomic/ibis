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
use ibis_api_client::{
    CLIENT,
    errors::{FrontendError, FrontendResultExt},
};
use ibis_database::common::{
    article::{Conflict, Edit},
    comment::Comment,
    notifications::{ApiNotification, ApiNotificationData},
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
                            n.iter()
                                .map(|notif| {
                                    use ApiNotificationData::*;
                                    let refresh_res = notifications;
                                    match &notif.data {
                                        EditConflict(c) => {
                                            EitherOf4::A(edit_conflict_view(notif, c, refresh_res))
                                        }
                                        ArticleCreated => {
                                            EitherOf4::B(article_view(notif, refresh_res))
                                        }
                                        Comment(c) => {
                                            EitherOf4::C(comment_view(notif, c, refresh_res))
                                        }
                                        Edit(e) => EitherOf4::D(edit_view(notif, e, refresh_res)),
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
    notif: &ApiNotification,
    conflict: &Conflict,
    refresh_res: NotificationsResource,
) -> impl IntoView {
    let href = format!(
        "{}/edit?conflict_id={}",
        article_path(&notif.article),
        conflict.id.0,
    );
    view! {
        <li class="py-2">
            <CardTitle notif=notif.clone() />
            <div>"Edit conflict: "{conflict.summary.clone()}</div>
            <CardActions href=href notif=notif.clone() refresh_res=refresh_res />
        </li>
    }
}

fn article_view(notif: &ApiNotification, refresh_res: NotificationsResource) -> impl IntoView {
    view! {
        <li class="py-2">
            <CardTitle notif=notif.clone() />
            <div>"New Article: "{article_title(&notif.article)}</div>
            <CardActions
                href=article_path(&notif.article)
                notif=notif.clone()
                refresh_res=refresh_res
            />
        </li>
    }
}

fn comment_view(
    notif: &ApiNotification,
    comment: &Comment,
    refresh_res: NotificationsResource,
) -> impl IntoView {
    view! {
        <li class="py-2">
            <CardTitle notif=notif.clone() />
            <div>"New comment: "{comment.content.clone()}</div>
            <CardActions
                href=comment_path(&comment, &notif.article)
                notif=notif.clone()
                refresh_res=refresh_res
            />
        </li>
    }
}

fn edit_view(
    notif: &ApiNotification,
    edit: &Edit,
    refresh_res: NotificationsResource,
) -> impl IntoView {
    view! {
        <li class="py-2">
            <CardTitle notif=notif.clone() />
            <div>"New edit: "{edit.summary.clone()}</div>
            <CardActions
                href=edit_path(&edit, &notif.article)
                notif=notif.clone()
                refresh_res=refresh_res
            />
        </li>
    }
}

#[component]
fn CardTitle(notif: ApiNotification) -> impl IntoView {
    view! {
        <div class="flex text-s">
            <span class="grow">{user_link(&notif.creator)}" - "{article_link(&notif.article)}</span>
            {time_ago(notif.published)}
        </div>
    }
}

#[component]
fn CardActions(
    href: String,
    notif: ApiNotification,
    refresh_res: NotificationsResource,
) -> impl IntoView {
    let id = notif.id;
    let click_mark_as_read = Action::new(move |_: &()| async move {
        CLIENT
            .article_notif_mark_as_read(id)
            .await
            .error_popup(|_| refresh_res.refetch());
    });
    let mark_as_read_action = move || {
        click_mark_as_read.dispatch(());
    };
    view! {
        <div class="mt-2 card-actions">
            <a class="btn btn-sm btn-outline" href=href title="View">
                <Icon icon=LINK />
            </a>
            <button
                class="btn btn-sm btn-outline"
                on:click=move |_| mark_as_read_action()
                title="Mark as read"
            >
                <Icon icon=CHECK />
            </button>
        </div>
    }
}
