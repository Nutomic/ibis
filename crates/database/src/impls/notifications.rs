use crate::{
    common::{
        article::{Article, Conflict, Edit},
        comment::Comment,
        newtypes::{
            ArticleId,
            ArticleNotifId,
            CommentId,
            ConflictId,
            EditId,
            LocalUserId,
            PersonId,
        },
        notifications::{ApiNotification, ApiNotificationData},
        user::{LocalUserView, Person},
    },
    error::BackendResult,
    impls::IbisContext,
    schema::{
        article,
        article_follow,
        comment,
        conflict,
        edit,
        instance_follow,
        local_user,
        notification,
        person,
    },
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

impl Notification {
    pub async fn list(
        user: &LocalUserView,
        context: &IbisContext,
    ) -> BackendResult<Vec<ApiNotification>> {
        let mut conn = context.db_pool.get()?;

        let article_notifications = notification::table
            .inner_join(article::table)
            .inner_join(person::table)
            .left_join(comment::table)
            .left_join(edit::table)
            .left_join(conflict::table)
            .filter(notification::local_user_id.eq(user.local_user.id))
            .order_by(notification::published.desc())
            .select((
                notification::all_columns,
                article::all_columns,
                person::all_columns,
                comment::all_columns.nullable(),
                edit::all_columns.nullable(),
                conflict::all_columns.nullable(),
            ))
            .get_results::<(
                Notification,
                Article,
                Person,
                Option<Comment>,
                Option<Edit>,
                Option<Conflict>,
            )>(&mut conn)?;

        Ok(article_notifications
            .into_iter()
            .map(|(notif, article, creator, comment, edit, conflict)| {
                use ApiNotificationData::*;
                let (published, data) = if let Some(c) = comment {
                    (c.published, Comment(c))
                } else if let Some(e) = edit {
                    (e.published, Edit(e))
                } else if let Some(c) = conflict {
                    (c.published, EditConflict {
                        conflict_id: c.id,
                        summary: c.summary,
                    })
                } else {
                    (article.published, ArticleCreated)
                };
                ApiNotification {
                    id: notif.id,
                    creator,
                    article,
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
        id: ArticleNotifId,
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

    pub fn notify_article(
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

        insert_into(notification::table)
            .values(&notifs)
            .on_conflict_do_nothing()
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
                conflict_id: None,
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
