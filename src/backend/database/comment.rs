use super::{
    schema::{comment, person},
    IbisContext,
};
use crate::{
    backend::utils::error::BackendResult,
    common::{
        comment::{DbComment, DbCommentView},
        newtypes::{ArticleId, CommentId, PersonId},
        user::DbPerson,
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
    pub ap_id: Option<ObjectId<DbComment>>,
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
    pub ap_id: Option<ObjectId<DbComment>>,
    pub updated: Option<DateTime<Utc>>,
}

impl DbComment {
    pub fn create(form: DbCommentInsertForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(comment::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn update(
        form: DbCommentUpdateForm,
        id: CommentId,
        context: &IbisContext,
    ) -> BackendResult<DbCommentView> {
        let mut conn = context.db_pool.get()?;
        let comment: DbComment = update(comment::table.find(id))
            .set(form)
            .get_result(conn.deref_mut())?;
        let creator = DbPerson::read(comment.creator_id, context)?;
        Ok(DbCommentView { comment, creator })
    }

    pub fn create_or_update(
        form: DbCommentInsertForm,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(comment::table)
            .values(&form)
            .on_conflict(comment::dsl::ap_id)
            .do_update()
            .set(&form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: CommentId, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(comment::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read_view(id: CommentId, context: &IbisContext) -> BackendResult<DbCommentView> {
        let mut conn = context.db_pool.get()?;
        let comment = comment::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?;
        let creator = DbPerson::read(comment.creator_id, context)?;
        Ok(DbCommentView { comment, creator })
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbComment>,
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
    ) -> BackendResult<Vec<DbCommentView>> {
        let mut conn = context.db_pool.get()?;
        let comments = comment::table
            .inner_join(person::table)
            .filter(comment::article_id.eq(article_id))
            .order_by(comment::published.desc())
            .get_results::<(DbComment, DbPerson)>(conn.deref_mut())?;

        // Clear content of deleted comments. comments themselves are returned
        // so that tree can be rendered.
        Ok(comments
            .into_iter()
            .map(|(mut comment, creator)| {
                if comment.deleted {
                    comment.content = String::new()
                };
                DbCommentView { comment, creator }
            })
            .collect())
    }
}
