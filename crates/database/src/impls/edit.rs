use super::notifications::Notification;
use crate::{
    DbUrl,
    common::{
        article::{Article, Edit, EditVersion, EditView},
        newtypes::{ArticleId, PersonId},
        user::LocalUserView,
    },
    error::BackendResult,
    impls::IbisContext,
    schema::{article, edit, person},
};
use chrono::{DateTime, Utc};
use diesel::{
    AsChangeset,
    BoolExpressionMethods,
    ExpressionMethods,
    Insertable,
    QueryDsl,
    RunQueryDsl,
    dsl::not,
    insert_into,
};
use diffy::create_patch;
use std::ops::DerefMut;
use url::Url;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = edit, check_for_backend(diesel::pg::Pg))]
pub struct DbEditForm {
    pub creator_id: PersonId,
    pub hash: EditVersion,
    pub ap_id: DbUrl,
    pub diff: String,
    pub summary: String,
    pub article_id: ArticleId,
    pub previous_version_id: EditVersion,
    pub published: DateTime<Utc>,
    pub pending: bool,
}

impl DbEditForm {
    pub fn new(
        original_article: &Article,
        creator_id: PersonId,
        updated_text: &str,
        summary: String,
        previous_version_id: EditVersion,
        pending: bool,
    ) -> BackendResult<Self> {
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
            pending,
        })
    }

    pub fn generate_ap_id(article: &Article, version: &EditVersion) -> BackendResult<DbUrl> {
        Ok(Url::parse(&format!("{}/{}", article.ap_id, version.hash()))?.into())
    }
}

impl Edit {
    pub fn create(form: &DbEditForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        let edit: Edit = insert_into(edit::table)
            .values(form)
            .on_conflict(edit::dsl::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?;

        Notification::notify_edit(&edit, context)?;
        Ok(edit)
    }

    pub fn read(version: &EditVersion, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(edit::table
            .filter(edit::dsl::hash.eq(version))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(ap_id: &DbUrl, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(edit::table
            .filter(edit::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn list_for_article(id: ArticleId, context: &IbisContext) -> BackendResult<Vec<Self>> {
        let mut conn = context.db_pool.get()?;
        Ok(edit::table
            .filter(edit::article_id.eq(id))
            .order(edit::published)
            .get_results(conn.deref_mut())?)
    }

    pub fn view(
        params: ViewEditParams,
        user: &Option<LocalUserView>,
        context: &IbisContext,
    ) -> BackendResult<Vec<EditView>> {
        let mut conn = context.db_pool.get()?;
        let person_id = user.as_ref().map(|u| u.person.id).unwrap_or(PersonId(-1));
        let query = edit::table
            .inner_join(article::table)
            .inner_join(person::table)
            // only the creator can view pending edits
            .filter(not(edit::pending).or(edit::creator_id.eq(person_id)))
            .into_boxed();

        let query = match params {
            ViewEditParams::PersonId(person_id) => query.filter(edit::creator_id.eq(person_id)),
            ViewEditParams::ArticleId(article_id) => query.filter(edit::article_id.eq(article_id)),
        };

        Ok(query.order(edit::published).get_results(conn.deref_mut())?)
    }
}

pub enum ViewEditParams {
    PersonId(PersonId),
    ArticleId(ArticleId),
}
