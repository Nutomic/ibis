use crate::database::article::DbArticle;
use crate::database::schema::edit;
use crate::error::MyResult;
use activitypub_federation::fetch::object_id::ObjectId;
use diesel::ExpressionMethods;
use diesel::{
    insert_into, AsChangeset, Identifiable, Insertable, PgConnection, QueryDsl, Queryable,
    RunQueryDsl, Selectable,
};
use diesel_derive_newtype::DieselNewType;
use diffy::create_patch;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha224};
use std::ops::DerefMut;
use std::sync::Mutex;
use url::Url;

/// Represents a single change to the article.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = edit, check_for_backend(diesel::pg::Pg))]
pub struct DbEdit {
    pub id: i32,
    pub ap_id: ObjectId<DbEdit>,
    pub diff: String,
    pub article_id: i32,
    pub version: EditVersion,
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
    pub local: bool,
}

impl DbEditForm {
    pub fn new(original_article: &DbArticle, updated_text: &str) -> MyResult<Self> {
        let diff = create_patch(&original_article.text, updated_text);
        let mut sha224 = Sha224::new();
        sha224.update(diff.to_bytes());
        let hash = format!("{:X}", sha224.finalize());
        let edit_id = Url::parse(&format!("{}/{}", original_article.ap_id, hash))?;
        Ok(DbEditForm {
            ap_id: edit_id.into(),
            diff: diff.to_string(),
            article_id: original_article.id,
            version: EditVersion(hash),
            local: true,
        })
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

    pub fn for_article(id: i32, conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        let mut conn = conn.lock().unwrap();
        Ok(edit::table
            .filter(edit::dsl::id.eq(id))
            .order_by(edit::dsl::id.asc())
            .get_results(conn.deref_mut())?)
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
