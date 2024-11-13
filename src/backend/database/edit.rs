use crate::{
    backend::{
        database::schema::{edit, person},
        error::MyResult,
        IbisData,
    },
    common::{
        newtypes::{ArticleId, PersonId},
        DbArticle,
        DbEdit,
        EditVersion,
        EditView,
    },
};
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use diesel::{insert_into, AsChangeset, ExpressionMethods, Insertable, QueryDsl, RunQueryDsl};
use diffy::create_patch;
use std::ops::DerefMut;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = edit, check_for_backend(diesel::pg::Pg))]
pub struct DbEditForm {
    pub creator_id: PersonId,
    pub hash: EditVersion,
    pub ap_id: ObjectId<DbEdit>,
    pub diff: String,
    pub summary: String,
    pub article_id: ArticleId,
    pub previous_version_id: EditVersion,
    pub published: DateTime<Utc>,
}

impl DbEditForm {
    pub fn new(
        original_article: &DbArticle,
        creator_id: PersonId,
        updated_text: &str,
        summary: String,
        previous_version_id: EditVersion,
    ) -> MyResult<Self> {
        let diff = create_patch(&original_article.text, updated_text);
        let version = EditVersion::new(&diff.to_string());
        let ap_id = Self::generate_ap_id(original_article, &version)?;
        Ok(DbEditForm {
            hash: version,
            ap_id,
            diff: diff.to_string(),
            creator_id,
            article_id: original_article.id,
            previous_version_id,
            summary,
            published: Utc::now(),
        })
    }

    pub fn generate_ap_id(
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
    pub fn create(form: &DbEditForm, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(insert_into(edit::table)
            .values(form)
            .on_conflict(edit::dsl::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(version: &EditVersion, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(edit::table
            .filter(edit::dsl::hash.eq(version))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(ap_id: &ObjectId<DbEdit>, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(edit::table
            .filter(edit::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    // TODO: create internal variant which doesnt return person?
    pub fn read_for_article(article: &DbArticle, data: &IbisData) -> MyResult<Vec<EditView>> {
        let mut conn = data.db_pool.get()?;
        Ok(edit::table
            .inner_join(person::table)
            .filter(edit::article_id.eq(article.id))
            .order(edit::published)
            .get_results(conn.deref_mut())?)
    }
}
