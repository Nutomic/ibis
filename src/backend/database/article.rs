use crate::{
    backend::{
        database::{
            schema::{article, article_follow, edit, instance},
            IbisContext,
        },
        federation::objects::edits_collection::DbEditCollection,
        utils::error::BackendResult,
    },
    common::{
        article::{DbArticle, DbArticleView, EditVersion},
        comment::DbComment,
        newtypes::{ArticleId, InstanceId},
        user::DbPerson,
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
    NullableExpressionMethods,
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

#[derive(Debug)]
pub enum ArticleViewQuery<'a> {
    Id(ArticleId),
    Name(&'a str, Option<String>),
}

impl Into<ArticleViewQuery<'_>> for ArticleId {
    fn into(self) -> ArticleViewQuery<'static> {
        ArticleViewQuery::Id(self)
    }
}
impl<'a> Into<ArticleViewQuery<'a>> for (&'a String, Option<String>) {
    fn into(self) -> ArticleViewQuery<'a> {
        ArticleViewQuery::Name(self.0, self.1)
    }
}

// TODO: get rid of unnecessary methods
impl DbArticle {
    pub fn edits_id(&self) -> BackendResult<CollectionId<DbEditCollection>> {
        Ok(CollectionId::parse(&format!("{}/edits", self.ap_id))?)
    }

    pub fn create(form: DbArticleForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(article::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn create_or_update(form: DbArticleForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(article::table)
            .values(&form)
            .on_conflict(article::dsl::ap_id)
            .do_update()
            .set(&form)
            .get_result(conn.deref_mut())?)
    }

    pub fn update_text(id: ArticleId, text: &str, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::text.eq(text))
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn update_protected(
        id: ArticleId,
        locked: bool,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::protected.eq(locked))
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn update_approved(
        id: ArticleId,
        approved: bool,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::approved.eq(approved))
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn delete(id: ArticleId, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::delete(article::dsl::article.find(id)).get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read(id: ArticleId, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(article::table
            .find(id)
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read_view<'a>(
        params: impl Into<ArticleViewQuery<'a>>,
        context: &IbisContext,
    ) -> BackendResult<DbArticleView> {
        let mut conn = context.db_pool.get()?;
        let mut query = article::table
            .inner_join(instance::table)
            .left_join(article_follow::table)
            .into_boxed();
        query = match dbg!(params.into()) {
            ArticleViewQuery::Id(id) => query.filter(article::id.eq(id)),
            ArticleViewQuery::Name(title, domain) => {
                query = query.filter(article::dsl::title.eq(title));
                if let Some(domain) = domain {
                    query.filter(instance::dsl::domain.eq(domain))
                } else {
                    query.filter(article::dsl::local.eq(true))
                }
            }
        };
        let (article, instance, following): (DbArticle, _, _) = query
            .select((
                article::all_columns,
                instance::all_columns,
                article_follow::person_id.nullable().is_not_null(),
            ))
            .get_result(conn.deref_mut())?;
        let comments = DbComment::read_for_article(article.id, context)?;
        let latest_version = article.latest_edit_version(context)?;
        Ok(DbArticleView {
            article,
            instance,
            comments,
            latest_version,
            following,
        })
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbArticle>,
        context: &IbisContext,
    ) -> BackendResult<Self> {
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
    ) -> BackendResult<Vec<Self>> {
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

    pub fn search(query: &str, context: &IbisContext) -> BackendResult<Vec<Self>> {
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

    pub fn latest_edit_version(&self, context: &IbisContext) -> BackendResult<EditVersion> {
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

    pub fn follow(
        article_id_: ArticleId,
        follower: &DbPerson,
        context: &IbisContext,
    ) -> BackendResult<()> {
        use article_follow::dsl::{article_id, person_id};
        let mut conn = context.db_pool.get()?;
        let form = (article_id.eq(article_id_), person_id.eq(follower.id));
        let rows = insert_into(article_follow::table)
            .values(form)
            .execute(conn.deref_mut())?;
        debug_assert_eq!(1, rows);
        Ok(())
    }
}
