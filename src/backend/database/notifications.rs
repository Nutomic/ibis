use super::{
    conflict::DbConflict,
    schema::{article, article_follow, article_notification, comment, conflict, person},
    IbisContext,
};
use crate::{
    backend::{api::check_is_admin, utils::error::BackendResult},
    common::{
        article::{ArticleNotificationKind, ArticleNotificationView, DbArticle},
        comment::CommentViewWithArticle,
        newtypes::{ArticleId, ArticleNotifId, LocalUserId},
        user::LocalUserView,
        Notification,
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

diesel::alias!(comment as parent_comment: DbComment);

impl Notification {
    pub(crate) async fn list(
        user: &LocalUserView,
        context: &Data<IbisContext>,
    ) -> BackendResult<Vec<Notification>> {
        let mut conn = context.db_pool.get()?;
        let mut notifications: Vec<Notification> = vec![];

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
        notifications.extend(conflicts.flatten().map(Notification::EditConflict));

        // comment replies
        let comment_replies = comment::table
            .inner_join(article::table)
            .inner_join(person::table.on(person::id.eq(comment::creator_id)))
            .inner_join(
                parent_comment.on(parent_comment
                    .field(comment::id)
                    .nullable()
                    .eq(comment::parent_id)),
            )
            .filter(parent_comment.field(comment::creator_id).eq(user.person.id))
            .filter(comment::creator_id.ne(user.person.id))
            .filter(not(comment::deleted))
            .filter(not(comment::read_by_parent_creator))
            .order_by(comment::published.desc())
            .select((
                comment::all_columns,
                person::all_columns,
                article::all_columns,
            ))
            .get_results::<CommentViewWithArticle>(conn.deref_mut())?
            .into_iter();
        notifications.extend(comment_replies.map(Notification::Reply));

        // new articles requiring approval
        if check_is_admin(user).is_ok() {
            let articles = article::table
                .group_by(article::dsl::id)
                .filter(article::dsl::approved.eq(false))
                .select(article::all_columns)
                .get_results(&mut conn)?
                .into_iter();
            notifications.extend(articles.map(Notification::ArticleApprovalRequired))
        }

        // new edits and comments for followed articles
        let article_notifications = article_notification::table
            .inner_join(article::table)
            .filter(article_notification::local_user_id.eq(user.local_user.id))
            .select((article::all_columns, article_notification::all_columns))
            .get_results::<(DbArticle, ArticleNotification)>(&mut conn)?;
        notifications.extend(
            article_notifications
                .into_iter()
                .map(|(article, notif)| ArticleNotificationView {
                    article,
                    id: notif.id,
                    kind: notif
                        .new_comments
                        .then_some(ArticleNotificationKind::Comment)
                        .unwrap_or(ArticleNotificationKind::Edit),
                    published: notif.published,
                })
                .map(Notification::ArticleNotification),
        );

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

        // comment replies
        let comment_replies = comment::table
            .inner_join(
                parent_comment.on(parent_comment
                    .field(comment::id)
                    .nullable()
                    .eq(comment::parent_id)),
            )
            .filter(parent_comment.field(comment::creator_id).eq(user.person.id))
            .filter(comment::creator_id.ne(user.person.id))
            .filter(not(comment::deleted))
            .filter(not(comment::read_by_parent_creator))
            .select(count(comment::id))
            .first::<i64>(conn.deref_mut())
            .unwrap_or(0);
        num += comment_replies;

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
        let article_notifications = article_notification::table
            .filter(article_notification::local_user_id.eq(user.local_user.id))
            .select(count(article_notification::id))
            .first::<i64>(conn.deref_mut())
            .unwrap_or(0);
        num += article_notifications;

        Ok(num)
    }
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = article_notification, check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)]
pub struct ArticleNotification {
    id: ArticleNotifId,
    local_user_id: LocalUserId,
    article_id: ArticleId,
    new_comments: bool,
    new_edits: bool,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = article_notification, check_for_backend(diesel::pg::Pg))]
struct ArticleNotificationInsertForm {
    local_user_id: LocalUserId,
    article_id: ArticleId,
    new_comments: Option<bool>,
    new_edits: Option<bool>,
}

impl ArticleNotification {
    pub fn mark_as_read(
        id: ArticleNotifId,
        user: &LocalUserView,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        delete(
            article_notification::table
                .filter(article_notification::id.eq(id))
                .filter(article_notification::local_user_id.eq(user.local_user.id)),
        )
        .execute(&mut conn)?;
        Ok(())
    }
    pub(super) fn notify_comment(
        article_id: ArticleId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        Self::notify(
            article_id,
            |local_user_id| ArticleNotificationInsertForm {
                local_user_id,
                article_id,
                new_comments: Some(true),
                new_edits: None,
            },
            context,
        )
    }

    pub(super) fn notify_edit(article_id: ArticleId, context: &IbisContext) -> BackendResult<()> {
        Self::notify(
            article_id,
            |local_user_id| ArticleNotificationInsertForm {
                local_user_id,
                article_id,
                new_comments: None,
                new_edits: Some(true),
            },
            context,
        )
    }

    fn notify<F>(article_id: ArticleId, map_fn: F, context: &IbisContext) -> BackendResult<()>
    where
        F: FnMut(LocalUserId) -> ArticleNotificationInsertForm,
    {
        let mut conn = context.db_pool.get()?;
        // get followers for this article
        let followers = article_follow::table
            .filter(article_follow::article_id.eq(article_id))
            .select(article_follow::local_user_id)
            .get_results(&mut conn)?;
        // create insert form with edit/comment it
        let notifications: Vec<_> = followers.into_iter().map(map_fn).collect();
        // insert all of them, generating at most one edit notification per user and article
        // (as well as a separate comment notification)
        insert_into(article_notification::table)
            .values(&notifications)
            .on_conflict_do_nothing()
            .execute(&mut conn)?;
        Ok(())
    }
}
