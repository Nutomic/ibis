use super::{
    notifications::Notification,
    schema::{comment, person},
    IbisContext,
};
use crate::{
    backend::utils::error::BackendResult,
    common::{
        comment::{Comment, CommentView},
        newtypes::{ArticleId, CommentId, PersonId},
        user::Person,
    },
};
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use diesel::{
    dsl::insert_into,
    update,
    AsChangeset,
    ExpressionMethods,
    Insertable,
    QueryDsl,
    RunQueryDsl,
};
use std::ops::DerefMut;

#[derive(Insertable, AsChangeset, Debug)]
#[diesel(table_name = comment, check_for_backend(diesel::pg::Pg))]
pub struct DbCommentInsertForm {
    pub creator_id: PersonId,
    pub article_id: ArticleId,
    pub parent_id: Option<CommentId>,
    pub content: String,
    pub depth: i32,
    pub ap_id: Option<ObjectId<Comment>>,
    pub local: bool,
    pub deleted: bool,
    pub published: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = comment, check_for_backend(diesel::pg::Pg))]
pub struct DbCommentUpdateForm {
    pub content: Option<String>,
    pub deleted: Option<bool>,
    pub ap_id: Option<ObjectId<Comment>>,
    pub updated: Option<DateTime<Utc>>,
}

impl Comment {
    pub fn create(form: DbCommentInsertForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        let comment: Comment = insert_into(comment::table)
            .values(&form)
            .on_conflict(comment::dsl::ap_id)
            .do_update()
            .set(&form)
            .get_result(conn.deref_mut())?;

        Notification::notify_comment(&comment, context)?;
        Ok(comment)
    }

    pub fn update(
        form: DbCommentUpdateForm,
        id: CommentId,
        context: &IbisContext,
    ) -> BackendResult<CommentView> {
        let mut conn = context.db_pool.get()?;
        let comment: Comment = update(comment::table.find(id))
            .set(form)
            .get_result(conn.deref_mut())?;
        let creator = Person::read(comment.creator_id, context)?;
        Ok(CommentView { comment, creator })
    }

    pub fn read(id: CommentId, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(comment::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read_view(id: CommentId, context: &IbisContext) -> BackendResult<CommentView> {
        let mut conn = context.db_pool.get()?;
        let comment = comment::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?;
        let creator = Person::read(comment.creator_id, context)?;
        Ok(CommentView { comment, creator })
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<Comment>,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(comment::table
            .filter(comment::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_for_article(
        article_id: ArticleId,
        context: &IbisContext,
    ) -> BackendResult<Vec<CommentView>> {
        let mut conn = context.db_pool.get()?;
        let comments = comment::table
            .inner_join(person::table)
            .filter(comment::article_id.eq(article_id))
            .order_by(comment::published.desc())
            .get_results::<CommentView>(conn.deref_mut())?;

        // Clear content of deleted comments. comments themselves are returned
        // so that tree can be rendered.
        Ok(comments
            .into_iter()
            .map(|mut view| {
                if view.comment.deleted {
                    view.comment.content = String::new()
                };
                view
            })
            .collect())
    }
}
