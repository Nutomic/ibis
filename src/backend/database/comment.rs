use super::{schema::comment, IbisData};
use crate::{
    backend::utils::error::MyResult,
    common::{
        comment::DbComment,
        newtypes::{ArticleId, CommentId, PersonId},
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
    pub fn create(form: DbCommentInsertForm, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        dbg!(&form);
        Ok(insert_into(comment::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn update(form: DbCommentUpdateForm, id: CommentId, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(update(comment::table.find(id))
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn create_or_update(form: DbCommentInsertForm, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(insert_into(comment::table)
            .values(&form)
            .on_conflict(comment::dsl::ap_id)
            .do_update()
            .set(&form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: CommentId, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(comment::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(ap_id: &ObjectId<DbComment>, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(comment::table
            .filter(comment::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }
    pub fn read_for_article(article_id: ArticleId, data: &IbisData) -> MyResult<Vec<Self>> {
        let mut conn = data.db_pool.get()?;
        Ok(comment::table
            .filter(comment::article_id.eq(article_id))
            .get_results::<Self>(conn.deref_mut())?)
    }
}
