use crate::database::schema::edit;
use crate::database::version::EditVersion;
use crate::database::DbArticle;
use crate::error::MyResult;
use activitypub_federation::fetch::object_id::ObjectId;
use diesel::ExpressionMethods;
use diesel::{
    insert_into, AsChangeset, Insertable, PgConnection, QueryDsl, Queryable, RunQueryDsl,
    Selectable,
};
use diffy::create_patch;
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::sync::Mutex;

/// Represents a single change to the article.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable)]
#[diesel(table_name = edit, check_for_backend(diesel::pg::Pg))]
pub struct DbEdit {
    // TODO: we could use hash as primary key, but that gives errors on forking because
    //       the same edit is used for multiple articles
    pub id: i32,
    /// UUID built from sha224 hash of diff
    pub hash: EditVersion,
    pub ap_id: ObjectId<DbEdit>,
    pub diff: String,
    pub article_id: i32,
    /// First edit of an article always has `EditVersion::default()` here
    pub previous_version_id: EditVersion,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = edit, check_for_backend(diesel::pg::Pg))]
pub struct DbEditForm {
    pub hash: EditVersion,
    pub ap_id: ObjectId<DbEdit>,
    pub diff: String,
    pub article_id: i32,
    pub previous_version_id: EditVersion,
}

impl DbEditForm {
    pub fn new(
        original_article: &DbArticle,
        updated_text: &str,
        previous_version_id: EditVersion,
    ) -> MyResult<Self> {
        let diff = create_patch(&original_article.text, updated_text);
        let version = EditVersion::new(&diff.to_string())?;
        let ap_id = Self::generate_ap_id(original_article, &version)?;
        Ok(DbEditForm {
            hash: version,
            ap_id,
            diff: diff.to_string(),
            article_id: original_article.id,
            previous_version_id,
        })
    }

    pub(crate) fn generate_ap_id(
        article: &DbArticle,
        version: &EditVersion,
    ) -> MyResult<ObjectId<DbEdit>> {
        Ok(ObjectId::parse(&format!(
            "{}/{}",
            article.ap_id,
            version.hash()
        ))?)
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
    pub fn read(version: &EditVersion, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(edit::table
            .filter(edit::dsl::hash.eq(version))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_for_article(
        article: &DbArticle,
        conn: &Mutex<PgConnection>,
    ) -> MyResult<Vec<Self>> {
        let mut conn = conn.lock().unwrap();
        Ok(edit::table
            .filter(edit::dsl::article_id.eq(article.id))
            .get_results(conn.deref_mut())?)
    }
}
