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
use diesel::{dsl::insert_into, AsChangeset, ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};
use std::ops::DerefMut;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = comment, check_for_backend(diesel::pg::Pg))]
pub struct DbCommentForm {
    pub creator_id: PersonId,
    pub article_id: ArticleId,
    pub parent_id: Option<CommentId>,
    pub content: String,
    pub ap_id: ObjectId<DbComment>,
    pub local: bool,
    pub deleted: bool,
    pub published: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

impl DbComment {
    pub fn create(form: DbCommentForm, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(insert_into(comment::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn create_or_update(form: DbCommentForm, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(insert_into(comment::table)
            .values(&form)
            .on_conflict(comment::dsl::ap_id)
            .do_update()
            .set(&form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(ap_id: &ObjectId<DbComment>, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(comment::table
            .filter(comment::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: CommentId, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(comment::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?)
    }
}
