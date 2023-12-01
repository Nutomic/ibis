use crate::database::schema::edit;
use crate::database::DbArticle;
use crate::error::MyResult;
use activitypub_federation::fetch::object_id::ObjectId;
use diesel::{
    insert_into, AsChangeset, Identifiable, Insertable, PgConnection, Queryable, RunQueryDsl,
    Selectable,
};
use diesel::{Associations, BelongingToDsl};
use diesel_derive_newtype::DieselNewType;
use diffy::create_patch;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha224};
use std::ops::DerefMut;
use std::sync::Mutex;

/// Represents a single change to the article.
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Queryable,
    Selectable,
    Identifiable,
    Associations,
)]
#[diesel(table_name = edit, check_for_backend(diesel::pg::Pg), belongs_to(DbArticle, foreign_key = article_id))]
pub struct DbEdit {
    pub id: i32,
    pub ap_id: ObjectId<DbEdit>,
    pub diff: String,
    pub article_id: i32,
    pub version: EditVersion,
    // TODO: could be an Option<DbEdit.id> instead
    pub previous_version: EditVersion,
    // TODO: there is already `local` field on article, do we need this?
    pub local: bool,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = edit, check_for_backend(diesel::pg::Pg))]
pub struct DbEditForm {
    pub ap_id: ObjectId<DbEdit>,
    pub diff: String,
    pub article_id: i32,
    pub version: EditVersion,
    pub previous_version: EditVersion,
    pub local: bool,
}

impl DbEditForm {
    pub fn new(
        original_article: &DbArticle,
        updated_text: &str,
        previous_version: EditVersion,
    ) -> MyResult<Self> {
        let diff = create_patch(&original_article.text, updated_text);
        let (ap_id, hash) = Self::generate_ap_id_and_hash(original_article, diff.to_bytes())?;
        Ok(DbEditForm {
            ap_id,
            diff: diff.to_string(),
            article_id: original_article.id,
            version: EditVersion(hash),
            previous_version,
            local: true,
        })
    }

    fn generate_ap_id_and_hash(
        article: &DbArticle,
        diff: Vec<u8>,
    ) -> MyResult<(ObjectId<DbEdit>, String)> {
        let mut sha224 = Sha224::new();
        sha224.update(diff);
        let hash = format!("{:X}", sha224.finalize());
        Ok((
            ObjectId::parse(&format!("{}/{}", article.ap_id, hash))?,
            hash,
        ))
    }
}

impl DbEdit {
    pub fn create(form: &DbEditForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(edit::table)
            .values(form)
            .on_conflict(edit::dsl::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn for_article(article: &DbArticle, conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        let mut conn = conn.lock().unwrap();
        Ok(DbEdit::belonging_to(&article).get_results(conn.deref_mut())?)
    }
    pub fn copy_to_local_fork(self, article: &DbArticle) -> MyResult<DbEditForm> {
        let (ap_id, _) =
            DbEditForm::generate_ap_id_and_hash(article, self.diff.clone().into_bytes())?;
        Ok(DbEditForm {
            ap_id,
            diff: self.diff,
            article_id: article.id,
            version: self.version,
            previous_version: self.previous_version,
            local: true,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, DieselNewType)]
pub struct EditVersion(pub String);

impl Default for EditVersion {
    fn default() -> Self {
        let sha224 = Sha224::new();
        let hash = format!("{:X}", sha224.finalize());
        EditVersion(hash)
    }
}
