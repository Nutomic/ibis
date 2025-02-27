use super::notifications::NotificationInsertForm;
use crate::{
    common::{
        article::{Conflict, EditVersion},
        newtypes::{ArticleId, ConflictId, PersonId},
        user::LocalUser,
    },
    error::BackendResult,
    impls::IbisContext,
    schema::{conflict, edit, local_user, notification},
};
use diesel::{ExpressionMethods, Insertable, QueryDsl, RunQueryDsl, delete, insert_into};
use std::ops::DerefMut;

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
        let conflict: Conflict = insert_into(conflict::table)
            .values(form)
            .get_result(conn.deref_mut())?;
        let local_user: LocalUser = local_user::table
            .filter(local_user::person_id.eq(conflict.creator_id))
            .get_result(&mut conn)?;

        let form = NotificationInsertForm {
            local_user_id: local_user.id,
            article_id: conflict.article_id,
            creator_id: conflict.creator_id,
            comment_id: None,
            edit_id: None,
            conflict_id: Some(conflict.id),
        };

        insert_into(notification::table)
            .values(&form)
            .execute(&mut conn)?;

        Ok(conflict)
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
