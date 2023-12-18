use crate::database::article::DbArticle;
use crate::database::edit::DbEdit;
use crate::database::schema::conflict;
use crate::database::version::EditVersion;
use crate::database::MyDataHandle;
use crate::error::MyResult;
use crate::federation::activities::submit_article_update;
use crate::utils::generate_article_version;
use activitypub_federation::config::Data;

use diesel::{
    delete, insert_into, Identifiable, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl,
    Selectable,
};
use diffy::{apply, merge, Patch};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::sync::Mutex;

/// A local only object which represents a merge conflict. It is created
/// when a local user edit conflicts with another concurrent edit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = conflict, check_for_backend(diesel::pg::Pg), belongs_to(DbArticle, foreign_key = article_id))]
pub struct DbConflict {
    pub id: EditVersion,
    pub diff: String,
    pub creator_id: i32,
    pub article_id: i32,
    pub previous_version_id: EditVersion,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiConflict {
    pub id: EditVersion,
    pub three_way_merge: String,
    pub article_id: i32,
    pub previous_version_id: EditVersion,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = conflict, check_for_backend(diesel::pg::Pg))]
pub struct DbConflictForm {
    pub id: EditVersion,
    pub diff: String,
    pub creator_id: i32,
    pub article_id: i32,
    pub previous_version_id: EditVersion,
}

impl DbConflict {
    pub fn create(form: &DbConflictForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(conflict::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn list(conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        let mut conn = conn.lock().unwrap();
        Ok(conflict::table.get_results(conn.deref_mut())?)
    }

    /// Delete a merge conflict after it is resolved.
    pub fn delete(id: EditVersion, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(delete(conflict::table.find(id)).get_result(conn.deref_mut())?)
    }

    pub async fn to_api_conflict(
        &self,
        data: &Data<MyDataHandle>,
    ) -> MyResult<Option<ApiConflict>> {
        let article = DbArticle::read(self.article_id, &data.db_connection)?;
        // Make sure to get latest version from origin so that all conflicts can be resolved
        let original_article = article.ap_id.dereference_forced(data).await?;

        // create common ancestor version
        let edits = DbEdit::read_for_article(&original_article, &data.db_connection)?;
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
                    self.previous_version_id.clone(),
                    &original_article,
                    self.creator_id,
                    data,
                )
                .await?;
                DbConflict::delete(self.id.clone(), &data.db_connection)?;
                Ok(None)
            }
            Err(three_way_merge) => {
                // there is a merge conflict, user needs to do three-way-merge
                Ok(Some(ApiConflict {
                    id: self.id.clone(),
                    three_way_merge,
                    article_id: original_article.id,
                    previous_version_id: original_article
                        .latest_edit_version(&data.db_connection)?,
                }))
            }
        }
    }
}
