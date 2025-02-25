use super::{
    conflict::DbConflict,
    schema::{article, article_follow, comment, conflict, edit, notification, person},
    IbisContext,
};
use crate::{
    backend::{api::check_is_admin, database::schema::local_user, utils::error::BackendResult},
    common::{
        article::{Article, Edit},
        comment::Comment,
        newtypes::{ArticleId, ArticleNotifId, CommentId, EditId, LocalUserId, PersonId},
        notifications::ApiNotification,
        user::{LocalUserView, Person},
    },
};
use activitypub_federation::config::Data;
use chrono::{DateTime, Utc};
use diesel::{
    dsl::*,
    ExpressionMethods,
    Insertable,
    JoinOnDsl,
    NullableExpressionMethods,
    QueryDsl,
    Queryable,
    RunQueryDsl,
    Selectable,
};
use futures::future::try_join_all;
use std::ops::DerefMut;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = notification, check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct Notification {
    id: ArticleNotifId,
    local_user_id: LocalUserId,
    article_id: ArticleId,
    creator_id: PersonId,
    comment_id: Option<CommentId>,
    edit_id: Option<EditId>,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = notification, check_for_backend(diesel::pg::Pg))]
struct NotificationInsertForm {
    local_user_id: LocalUserId,
    article_id: ArticleId,
    creator_id: PersonId,
    comment_id: Option<CommentId>,
    edit_id: Option<EditId>,
}

impl Notification {
    pub(crate) async fn list(
        user: &LocalUserView,
        context: &Data<IbisContext>,
    ) -> BackendResult<Vec<ApiNotification>> {
        let mut conn = context.db_pool.get()?;
        let mut notifications: Vec<ApiNotification> = vec![];

        // edit conflicts
        let conflicts: Vec<DbConflict> = conflict::table
            .filter(conflict::dsl::creator_id.eq(user.person.id))
            .get_results(conn.deref_mut())?;
        let conflicts = try_join_all(conflicts.into_iter().map(|c| {
            let data = context.reset_request_count();
            async move { c.to_api_conflict(false, &data).await }
        }))
        .await?
        .into_iter();
        notifications.extend(conflicts.flatten().map(ApiNotification::EditConflict));

        // new articles requiring approval
        if check_is_admin(user).is_ok() {
            let articles = article::table
                .group_by(article::dsl::id)
                .filter(article::dsl::approved.eq(false))
                .select(article::all_columns)
                .get_results(&mut conn)?
                .into_iter();
            notifications.extend(articles.map(ApiNotification::ArticleApprovalRequired))
        }

        // new edits and comments for followed articles
        let article_notifications = notification::table
            .inner_join(article::table)
            .inner_join(person::table)
            .left_join(comment::table)
            .left_join(edit::table)
            .filter(notification::local_user_id.eq(user.local_user.id))
            .select((
                notification::all_columns,
                article::all_columns,
                person::all_columns,
                comment::all_columns.nullable(),
                edit::all_columns.nullable(),
            ))
            .get_results::<(Notification, Article, Person, Option<Comment>, Option<Edit>)>(
                &mut conn,
            )?;
        notifications.extend(article_notifications.into_iter().flat_map(
            |(notif, article, creator, comment, edit)| {
                if let Some(c) = comment {
                    Some(ApiNotification::Comment(notif.id, c, creator, article))
                } else {
                    edit.map(|e| ApiNotification::Edit(notif.id, e, creator, article))
                }
            },
        ));

        notifications.sort_by(|a, b| b.published().cmp(a.published()));
        Ok(notifications)
    }

    pub fn count(user: &LocalUserView, context: &Data<IbisContext>) -> BackendResult<i64> {
        let mut conn = context.db_pool.get()?;
        let mut num = 0;
        // edit conflicts
        let conflicts = conflict::table
            .filter(conflict::dsl::creator_id.eq(user.person.id))
            .select(count(conflict::id))
            .first::<i64>(conn.deref_mut())
            .unwrap_or(0);
        num += conflicts;

        // new articles requiring approval
        if check_is_admin(user).is_ok() {
            let articles = article::table
                .group_by(article::dsl::id)
                .filter(article::dsl::approved.eq(false))
                .select(count(article::id))
                .first::<i64>(conn.deref_mut())
                .unwrap_or(0);
            num += articles;
        }

        // new edits and comments for followed articles
        let article_notifications = notification::table
            .filter(notification::local_user_id.eq(user.local_user.id))
            .select(count(notification::id))
            .first::<i64>(conn.deref_mut())
            .unwrap_or(0);
        num += article_notifications;

        Ok(num)
    }

    pub fn mark_as_read(
        id: ArticleNotifId,
        user: &LocalUserView,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        delete(
            notification::table
                .filter(notification::id.eq(id))
                .filter(notification::local_user_id.eq(user.local_user.id)),
        )
        .execute(&mut conn)?;
        Ok(())
    }

    pub(super) fn notify_comment(comment: &Comment, context: &IbisContext) -> BackendResult<()> {
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
            },
            context,
        )?;

        Ok(())
    }

    pub(super) fn notify_edit(edit: &Edit, context: &IbisContext) -> BackendResult<()> {
        Self::notify(
            edit.article_id,
            edit.creator_id,
            |local_user_id| NotificationInsertForm {
                local_user_id,
                article_id: edit.article_id,
                creator_id: edit.creator_id,
                comment_id: None,
                edit_id: Some(edit.id),
            },
            context,
        )
    }

    fn notify<F>(
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
        insert_into(notification::table)
            .values(&notifs)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(())
    }
}
