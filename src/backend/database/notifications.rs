use super::{
    conflict::DbConflict,
    schema::{article, comment, conflict, person},
    IbisContext,
};
use crate::{
    backend::{api::check_is_admin, utils::error::BackendResult},
    common::{comment::CommentViewWithArticle, user::LocalUserView, Notification},
};
use activitypub_federation::config::Data;
use diesel::{
    dsl::{count, not},
    ExpressionMethods,
    JoinOnDsl,
    NullableExpressionMethods,
    QueryDsl,
    RunQueryDsl,
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
        notifications.sort_by(|a, b| a.published().cmp(b.published()));

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

        Ok(num)
    }
}
