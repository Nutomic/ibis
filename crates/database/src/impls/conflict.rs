use crate::{
    common::{
        article::EditVersion,
        newtypes::{ArticleId, ConflictId, PersonId},
    },
    error::BackendResult,
    impls::IbisContext,
    schema::{conflict, edit},
};
use chrono::{DateTime, Utc};
use diesel::{
    delete,
    insert_into,
    ExpressionMethods,
    Identifiable,
    Insertable,
    QueryDsl,
    Queryable,
    RunQueryDsl,
    Selectable,
};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

/// A local only object which represents a merge conflict. It is created
/// when a local user edit conflicts with another concurrent edit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = conflict, check_for_backend(diesel::pg::Pg), belongs_to(DbArticle, foreign_key = article_id))]
pub struct Conflict {
    pub id: ConflictId,
    pub hash: EditVersion,
    pub diff: String,
    pub summary: String,
    pub creator_id: PersonId,
    pub article_id: ArticleId,
    pub previous_version_id: EditVersion,
    pub published: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = conflict, check_for_backend(diesel::pg::Pg))]
pub struct DbConflictForm {
    pub hash: EditVersion,
    pub diff: String,
    pub summary: String,
    pub creator_id: PersonId,
    pub article_id: ArticleId,
    pub previous_version_id: EditVersion,
}

impl Conflict {
    pub fn create(form: &DbConflictForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(conflict::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(
        id: ConflictId,
        person_id: PersonId,
        context: &IbisContext,
    ) -> BackendResult<Conflict> {
        let mut conn = context.db_pool.get()?;
        Ok(conflict::table
            .find(id)
            .filter(conflict::dsl::creator_id.eq(person_id))
            .get_result(conn.deref_mut())?)
    }

    /// Delete merge conflict which was created by specific user
    pub fn delete(
        id: ConflictId,
        creator_id: PersonId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        let conflict: Self = delete(
            conflict::table
                .filter(conflict::dsl::creator_id.eq(creator_id))
                .find(id),
        )
        .get_result(conn.deref_mut())?;
        delete(
            edit::table
                .filter(edit::dsl::creator_id.eq(creator_id))
                .filter(edit::dsl::hash.eq(conflict.hash)),
        )
        .execute(conn.deref_mut())?;
        Ok(())
    }
}
