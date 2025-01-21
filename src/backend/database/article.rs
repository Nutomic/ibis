use crate::{
    backend::{
        database::{
            schema::{article, edit, instance},
            IbisContext,
        },
        federation::objects::edits_collection::DbEditCollection,
        utils::error::MyResult,
    },
    common::{
        article::{DbArticle, DbArticleView, EditVersion},
        comment::DbComment,
        instance::DbInstance,
        newtypes::{ArticleId, InstanceId},
    },
};
use activitypub_federation::fetch::{collection_id::CollectionId, object_id::ObjectId};
use diesel::{
    dsl::max,
    insert_into,
    AsChangeset,
    BoolExpressionMethods,
    ExpressionMethods,
    Insertable,
    PgTextExpressionMethods,
    QueryDsl,
    RunQueryDsl,
};
use std::ops::DerefMut;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct DbArticleForm {
    pub title: String,
    pub text: String,
    pub ap_id: ObjectId<DbArticle>,
    pub instance_id: InstanceId,
    pub local: bool,
    pub protected: bool,
    pub approved: bool,
}

// TODO: get rid of unnecessary methods
impl DbArticle {
    pub fn edits_id(&self) -> MyResult<CollectionId<DbEditCollection>> {
        Ok(CollectionId::parse(&format!("{}/edits", self.ap_id))?)
    }

    pub fn create(form: DbArticleForm, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(article::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn create_or_update(form: DbArticleForm, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(article::table)
            .values(&form)
            .on_conflict(article::dsl::ap_id)
            .do_update()
            .set(&form)
            .get_result(conn.deref_mut())?)
    }

    pub fn update_text(id: ArticleId, text: &str, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::text.eq(text))
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn update_protected(id: ArticleId, locked: bool, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::protected.eq(locked))
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn update_approved(id: ArticleId, approved: bool, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::approved.eq(approved))
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn delete(id: ArticleId, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::delete(article::dsl::article.find(id)).get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read(id: ArticleId, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(article::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read_view(id: ArticleId, context: &IbisContext) -> MyResult<DbArticleView> {
        let mut conn = context.db_pool.get()?;
        let query = article::table
            .find(id)
            .inner_join(instance::table)
            .into_boxed();
        let (article, instance): (DbArticle, DbInstance) = query.get_result(conn.deref_mut())?;
        let comments = DbComment::read_for_article(article.id, context)?;
        let latest_version = article.latest_edit_version(context)?;
        Ok(DbArticleView {
            article,
            instance,
            comments,
            latest_version,
        })
    }

    pub fn read_view_title(
        title: &str,
        domain: Option<String>,
        context: &IbisContext,
    ) -> MyResult<DbArticleView> {
        let mut conn = context.db_pool.get()?;
        let (article, instance): (DbArticle, DbInstance) = {
            let query = article::table
                .inner_join(instance::table)
                .filter(article::dsl::title.eq(title))
                .into_boxed();
            let query = if let Some(domain) = domain {
                query.filter(instance::dsl::domain.eq(domain))
            } else {
                query.filter(article::dsl::local.eq(true))
            };
            query.get_result(conn.deref_mut())?
        };
        let comments = DbComment::read_for_article(article.id, context)?;
        let latest_version = article.latest_edit_version(context)?;
        Ok(DbArticleView {
            article,
            instance,
            comments,
            latest_version,
        })
    }

    pub fn read_from_ap_id(ap_id: &ObjectId<DbArticle>, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(article::table
            .filter(article::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    /// Read all articles, ordered by most recently edited first.
    ///
    /// TODO: Should get rid of only_local param and rely on instance_id
    pub fn read_all(
        only_local: Option<bool>,
        instance_id: Option<InstanceId>,
        context: &IbisContext,
    ) -> MyResult<Vec<Self>> {
        let mut conn = context.db_pool.get()?;
        let mut query = article::table
            .inner_join(edit::table)
            .inner_join(instance::table)
            .filter(article::dsl::approved.eq(true))
            .group_by(article::dsl::id)
            .order_by(max(edit::dsl::published).desc())
            .select(article::all_columns)
            .into_boxed();

        if let Some(true) = only_local {
            query = query.filter(article::dsl::local.eq(true));
        }
        if let Some(instance_id) = instance_id {
            query = query.filter(instance::dsl::id.eq(instance_id));
        }
        Ok(query.get_results(&mut conn)?)
    }

    pub fn search(query: &str, context: &IbisContext) -> MyResult<Vec<Self>> {
        let mut conn = context.db_pool.get()?;
        let replaced = query
            .replace('%', "\\%")
            .replace('_', "\\_")
            .replace(' ', "%");
        let replaced = format!("%{replaced}%");
        Ok(article::table
            .filter(
                article::dsl::title
                    .ilike(&replaced)
                    .or(article::dsl::text.ilike(&replaced)),
            )
            .get_results(conn.deref_mut())?)
    }

    pub fn latest_edit_version(&self, context: &IbisContext) -> MyResult<EditVersion> {
        let mut conn = context.db_pool.get()?;
        let latest_version: Option<EditVersion> = edit::table
            .filter(edit::dsl::article_id.eq(self.id))
            .order_by(edit::dsl::id.desc())
            .limit(1)
            .select(edit::dsl::hash)
            .get_result(conn.deref_mut())
            .ok();
        match latest_version {
            Some(latest_version) => Ok(latest_version),
            None => Ok(EditVersion::default()),
        }
    }

    pub fn list_approval_required(context: &IbisContext) -> MyResult<Vec<Self>> {
        let mut conn = context.db_pool.get()?;
        let query = article::table
            .group_by(article::dsl::id)
            .filter(article::dsl::approved.eq(false))
            .select(article::all_columns)
            .into_boxed();

        Ok(query.get_results(&mut conn)?)
    }
}
