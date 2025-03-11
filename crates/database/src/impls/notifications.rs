use crate::{
    common::{
        article::{Article, Conflict, Edit},
        comment::Comment,
        newtypes::{
            ArticleId,
            CommentId,
            ConflictId,
            EditId,
            LocalUserId,
            NotificationId,
            PersonId,
        },
        notifications::{ApiNotification, ApiNotificationData},
        user::{LocalUser, LocalUserView, Person},
    },
    email::notification::send_notification_email,
    error::BackendResult,
    impls::IbisContext,
};
use chrono::{DateTime, Utc};
use diesel::{
    ExpressionMethods,
    Insertable,
    JoinOnDsl,
    NullableExpressionMethods,
    QueryDsl,
    Queryable,
    RunQueryDsl,
    Selectable,
    dsl::*,
};
use ibis_database_schema::{
    article,
    article_follow,
    comment,
    conflict,
    edit,
    instance_follow,
    local_user,
    notification,
    person,
};
use std::ops::DerefMut;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = notification, check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Notification {
    pub(crate) id: NotificationId,
    local_user_id: LocalUserId,
    article_id: ArticleId,
    creator_id: PersonId,
    comment_id: Option<CommentId>,
    edit_id: Option<EditId>,
    pub published: DateTime<Utc>,
    conflict_id: Option<ConflictId>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = notification, check_for_backend(diesel::pg::Pg))]
pub(crate) struct NotificationInsertForm {
    pub local_user_id: LocalUserId,
    pub article_id: ArticleId,
    pub creator_id: PersonId,
    pub comment_id: Option<CommentId>,
    pub edit_id: Option<EditId>,
    pub conflict_id: Option<ConflictId>,
}

#[derive(Queryable, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NotificationData {
    pub(crate) notification: Notification,
    pub(crate) article: Article,
    pub(crate) creator: Person,
    pub(crate) local_user: LocalUser,
    pub(crate) comment: Option<Comment>,
    pub(crate) edit: Option<Edit>,
    pub(crate) conflict: Option<Conflict>,
}

impl Notification {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub(crate) fn joins() -> _ {
        notification::table
            .inner_join(article::table)
            .inner_join(person::table)
            .inner_join(local_user::table)
            .left_join(comment::table)
            .left_join(edit::table)
            .left_join(conflict::table)
    }

    pub(crate) fn read_data(
        id: NotificationId,
        context: &IbisContext,
    ) -> BackendResult<NotificationData> {
        let mut conn = context.db_pool.get()?;
        Ok(Notification::joins()
            .filter(notification::id.eq(id))
            .get_result(&mut conn)?)
    }
    pub async fn list(
        user: &LocalUserView,
        context: &IbisContext,
    ) -> BackendResult<Vec<ApiNotification>> {
        let mut conn = context.db_pool.get()?;

        let article_notifications = Self::joins()
            .filter(notification::local_user_id.eq(user.local_user.id))
            .order_by(notification::published.desc())
            .get_results::<NotificationData>(&mut conn)?;

        Ok(article_notifications
            .into_iter()
            .map(|n| {
                use ApiNotificationData::*;
                let (published, data) = if let Some(c) = n.comment {
                    (c.published, Comment(c))
                } else if let Some(e) = n.edit {
                    (e.published, Edit(e))
                } else if let Some(c) = n.conflict {
                    (
                        c.published,
                        EditConflict {
                            conflict_id: c.id,
                            summary: c.summary,
                        },
                    )
                } else {
                    (n.article.published, ArticleCreated)
                };
                ApiNotification {
                    id: n.notification.id,
                    creator: n.creator,
                    article: n.article,
                    published,
                    data,
                }
            })
            .collect())
    }

    pub fn count(user: &LocalUserView, context: &IbisContext) -> BackendResult<i64> {
        let mut conn = context.db_pool.get()?;
        let mut num = 0;

        let article_notifications = notification::table
            .filter(notification::local_user_id.eq(user.local_user.id))
            .select(count(notification::id))
            .first::<i64>(conn.deref_mut())
            .unwrap_or(0);
        num += article_notifications;

        Ok(num)
    }

    pub fn mark_as_read(
        id: NotificationId,
        user: &LocalUserView,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        let notif: Notification = delete(
            notification::table
                .filter(notification::id.eq(id))
                .filter(notification::local_user_id.eq(user.local_user.id)),
        )
        .returning(notification::all_columns)
        .get_result(&mut conn)?;

        // if this is a conflict, delete the conflict as well
        if let Some(conflict_id) = notif.conflict_id {
            delete(
                conflict::table
                    .filter(conflict::id.eq(conflict_id))
                    .filter(conflict::creator_id.eq(user.person.id)),
            )
            .execute(&mut conn)?;
        }
        Ok(())
    }

    pub async fn notify_article(
        article: &Article,
        creator_id: PersonId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        let followers = instance_follow::table
            .inner_join(person::table.inner_join(local_user::table))
            .filter(instance_follow::instance_id.eq(article.instance_id))
            .select((local_user::person_id, local_user::id))
            .get_results::<(PersonId, LocalUserId)>(&mut conn)?;
        let notifs: Vec<_> = followers
            .into_iter()
            // exclude creator so he doesnt get notified about his own edit/comment
            .flat_map(|(person_id, local_user_id)| {
                if person_id != creator_id {
                    Some(local_user_id)
                } else {
                    None
                }
            })
            .map(|local_user_id| NotificationInsertForm {
                local_user_id,
                article_id: article.id,
                creator_id,
                comment_id: None,
                edit_id: None,
                conflict_id: None,
            })
            .collect();

        let notifs = insert_into(notification::table)
            .values(&notifs)
            .on_conflict_do_nothing()
            .get_results(&mut conn)?;
        send_notification_email(notifs, context).await?;
        Ok(())
    }

    pub async fn notify_comment(comment: &Comment, context: &IbisContext) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;

        // notify author of parent comment
        {
            diesel::alias!(comment as parent_comment: DbComment);
            let parent_comment_creator_id: Option<LocalUserId> = comment::table
                .find(comment.id)
                .left_join(
                    parent_comment.on(parent_comment
                        .field(comment::id)
                        .nullable()
                        .eq(comment::parent_id)),
                )
                .left_join(
                    local_user::table.on(parent_comment
                        .field(comment::creator_id)
                        .eq(local_user::person_id)),
                )
                .select(local_user::id.nullable())
                .get_result(conn.deref_mut())?;
            if let Some(local_user_id) = parent_comment_creator_id {
                let form = NotificationInsertForm {
                    local_user_id,
                    article_id: comment.article_id,
                    creator_id: comment.creator_id,
                    comment_id: Some(comment.id),
                    edit_id: None,
                    conflict_id: None,
                };
                insert_into(notification::table)
                    .values(&form)
                    .on_conflict_do_nothing()
                    .execute(&mut conn)?;
            }
        }

        // notify users who subscribed to article
        Self::notify(
            comment.article_id,
            comment.creator_id,
            |local_user_id| NotificationInsertForm {
                local_user_id,
                article_id: comment.article_id,
                creator_id: comment.creator_id,
                comment_id: Some(comment.id),
                edit_id: None,
                conflict_id: None,
            },
            context,
        )
        .await?;

        Ok(())
    }

    pub async fn notify_edit(edit: &Edit, context: &IbisContext) -> BackendResult<()> {
        Self::notify(
            edit.article_id,
            edit.creator_id,
            |local_user_id| NotificationInsertForm {
                local_user_id,
                article_id: edit.article_id,
                creator_id: edit.creator_id,
                comment_id: None,
                edit_id: Some(edit.id),
                conflict_id: None,
            },
            context,
        )
        .await?;
        Ok(())
    }

    async fn notify<F>(
        article_id: ArticleId,
        creator_id: PersonId,
        map_fn: F,
        context: &IbisContext,
    ) -> BackendResult<()>
    where
        F: FnMut(LocalUserId) -> NotificationInsertForm,
    {
        let mut conn = context.db_pool.get()?;
        // get followers for this article
        let followers = article_follow::table
            .inner_join(local_user::table)
            .filter(article_follow::article_id.eq(article_id))
            .select((local_user::person_id, local_user::id))
            .get_results::<(PersonId, LocalUserId)>(&mut conn)?;
        // create insert form with edit/comment it
        let notifs: Vec<_> = followers
            .into_iter()
            // exclude creator so he doesnt get notified about his own edit/comment
            .flat_map(|(person_id, local_user_id)| {
                if person_id != creator_id {
                    Some(local_user_id)
                } else {
                    None
                }
            })
            .map(map_fn)
            .collect();
        // insert all of them
        let notifs = insert_into(notification::table)
            .values(&notifs)
            .on_conflict_do_nothing()
            .get_results(&mut conn)?;
        send_notification_email(notifs, context).await?;
        Ok(())
    }
}
