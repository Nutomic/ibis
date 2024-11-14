use crate::{
    backend::{
        database::{schema::conflict, IbisData},
        error::MyResult,
        federation::activities::submit_article_update,
        utils::generate_article_version,
    },
    common::{
        newtypes::{ArticleId, ConflictId, PersonId},
        ApiConflict,
        DbArticle,
        DbEdit,
        DbPerson,
        EditVersion,
    },
};
use activitypub_federation::config::Data;
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
use diffy::{apply, merge, Patch};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;

/// A local only object which represents a merge conflict. It is created
/// when a local user edit conflicts with another concurrent edit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = conflict, check_for_backend(diesel::pg::Pg), belongs_to(DbArticle, foreign_key = article_id))]
pub struct DbConflict {
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

impl DbConflict {
    pub fn create(form: &DbConflictForm, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(insert_into(conflict::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn list(person: &DbPerson, data: &IbisData) -> MyResult<Vec<Self>> {
        let mut conn = data.db_pool.get()?;
        Ok(conflict::table
            .filter(conflict::dsl::creator_id.eq(person.id))
            .get_results(conn.deref_mut())?)
    }

    /// Delete merge conflict which was created by specific user
    pub fn delete(id: ConflictId, creator_id: PersonId, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(delete(
            conflict::table
                .filter(conflict::dsl::creator_id.eq(creator_id))
                .find(id),
        )
        .get_result(conn.deref_mut())?)
    }

    pub async fn to_api_conflict(&self, data: &Data<IbisData>) -> MyResult<Option<ApiConflict>> {
        let article = DbArticle::read_view(self.article_id, data)?;
        // Make sure to get latest version from origin so that all conflicts can be resolved
        let original_article = article.article.ap_id.dereference_forced(data).await?;

        // create common ancestor version
        let edits = DbEdit::read_for_article(&original_article, data)?;
        let ancestor = generate_article_version(&edits, &self.previous_version_id)?;

        let patch = Patch::from_str(&self.diff)?;
        // apply self.diff to ancestor to get `ours`
        let ours = apply(&ancestor, &patch)?;
        match merge(&ancestor, &ours, &original_article.text) {
            Ok(new_text) => {
                // patch applies cleanly so we are done
                // federate the change
                submit_article_update(
                    new_text,
                    self.summary.clone(),
                    self.previous_version_id.clone(),
                    &original_article,
                    self.creator_id,
                    data,
                )
                .await?;
                DbConflict::delete(self.id, self.creator_id, data)?;
                Ok(None)
            }
            Err(three_way_merge) => {
                // there is a merge conflict, user needs to do three-way-merge
                Ok(Some(ApiConflict {
                    id: self.id,
                    hash: self.hash.clone(),
                    three_way_merge,
                    summary: self.summary.clone(),
                    article: original_article.clone(),
                    previous_version_id: original_article.latest_edit_version(data)?,
                    published: self.published,
                }))
            }
        }
    }
}
