use crate::{
    backend::{
        database::{
            schema::{article, article_follow, article_notification, edit, instance},
            IbisContext,
        },
        federation::objects::edits_collection::DbEditCollection,
        utils::error::BackendResult,
    },
    common::{
        article::{DbArticle, DbArticleView, EditVersion},
        comment::DbComment,
        newtypes::{ArticleId, CommentId, EditId, InstanceId, PersonId},
        user::DbPerson,
    },
};
use activitypub_federation::fetch::{collection_id::CollectionId, object_id::ObjectId};
use diesel::{
    dsl::{delete, max},
    insert_into, AsChangeset, BoolExpressionMethods, ExpressionMethods, Insertable,
    NullableExpressionMethods, PgTextExpressionMethods, QueryDsl, Queryable, RunQueryDsl,
    Selectable,
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

impl From<ArticleId> for ArticleViewQuery<'_> {
    fn from(val: ArticleId) -> Self {
        ArticleViewQuery::Id(val)
    }
}
impl<'a> From<(&'a String, Option<String>)> for ArticleViewQuery<'a> {
    fn from(val: (&'a String, Option<String>)) -> Self {
        ArticleViewQuery::Name(val.0, val.1)
    }
}

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
        query = match params.into() {
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
        insert_into(article_follow::table)
            .values(form)
            .execute(conn.deref_mut())?;
        Ok(())
    }

    pub fn unfollow(
        article_id_: ArticleId,
        follower: &DbPerson,
        context: &IbisContext,
    ) -> BackendResult<()> {
        use article_follow::dsl::{article_id, person_id};
        let mut conn = context.db_pool.get()?;
        delete(
            article_follow::table.filter(article_id.eq(article_id_).and(person_id.eq(follower.id))),
        )
        .execute(conn.deref_mut())?;
        Ok(())
    }
}

#[cfg_attr(feature = "ssr", derive(Queryable, Selectable))]
#[cfg_attr(feature = "ssr", diesel(table_name = article_notification, check_for_backend(diesel::pg::Pg), belongs_to(DbInstance, foreign_key = instance_id)))]
pub struct ArticleNotification {
    id: i32,
    person_id: PersonId,
    comment_id: Option<CommentId>,
    edit_id: Option<EditId>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = article_notification, check_for_backend(diesel::pg::Pg))]
struct ArticleNotificationInsertForm {
    person_id: PersonId,
    comment_id: Option<CommentId>,
    edit_id: Option<EditId>,
}

impl ArticleNotification {
    pub(super) fn new_comment(
        article_id: ArticleId,
        comment_id: CommentId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let followers = ArticleNotification::article_followers(article_id, context)?;
        let notifications = followers
            .into_iter()
            .map(|f| ArticleNotificationInsertForm {
                person_id: f,
                comment_id: Some(comment_id),
                edit_id: None,
            })
            .collect();
        ArticleNotification::insert(notifications, context)?;
        Ok(())
    }

    pub(super) fn new_edit(
        article_id: ArticleId,
        edit_id: EditId,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let followers = ArticleNotification::article_followers(article_id, context)?;
        let notifications = followers
            .into_iter()
            .map(|f| ArticleNotificationInsertForm {
                person_id: f,
                comment_id: None,
                edit_id: Some(edit_id),
            })
            .collect();
        ArticleNotification::insert(notifications, context)?;
        Ok(())
    }

    fn article_followers(
        article_id: ArticleId,
        context: &IbisContext,
    ) -> BackendResult<Vec<PersonId>> {
        let mut conn = context.db_pool.get()?;
        Ok(article_follow::table
            .filter(article_follow::article_id.eq(article_id))
            .select(article_follow::person_id)
            .get_results(&mut conn)?)
    }

    fn insert(
        notifications: Vec<ArticleNotificationInsertForm>,
        context: &IbisContext,
    ) -> BackendResult<()> {
        let mut conn = context.db_pool.get()?;
        insert_into(article_notification::table)
            .values(notifications)
            .execute(&mut conn)?;
        Ok(())
    }
}
