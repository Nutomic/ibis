use super::{is_conflict, notifications::Notification};
use crate::{
    DbUrl,
    common::{
        article::{Article, ArticleView, EditVersion},
        comment::Comment,
        newtypes::{ArticleId, InstanceId, PersonId},
        user::LocalUserView,
    },
    error::BackendResult,
    impls::IbisContext,
};
use chrono::{DateTime, Utc};
use diesel::{
    AsChangeset,
    BoolExpressionMethods,
    ExpressionMethods,
    Insertable,
    JoinOnDsl,
    NullableExpressionMethods,
    PgTextExpressionMethods,
    QueryDsl,
    RunQueryDsl,
    dsl::{delete, max, not, now, update},
    insert_into,
};
use ibis_database_schema::{article, article_follow, edit, instance};
use std::ops::DerefMut;
use url::Url;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct DbArticleForm {
    pub title: String,
    pub text: String,
    pub ap_id: DbUrl,
    pub instance_id: InstanceId,
    pub local: bool,
    pub protected: bool,
    pub updated: DateTime<Utc>,
    pub pending: bool,
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

impl Article {
    pub fn edits_id(&self) -> BackendResult<DbUrl> {
        Ok(Url::parse(&format!("{}/edits", self.ap_id))?.into())
    }

    pub async fn create(
        form: DbArticleForm,
        creator_id: PersonId,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        let article = insert_into(article::table)
            .values(form)
            .get_result::<Self>(conn.deref_mut())?;

        Notification::notify_article(&article, creator_id, context).await?;
        Ok(article)
    }

    pub async fn create_or_update(
        form: DbArticleForm,
        creator_id: PersonId,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        let article = insert_into(article::table)
            .values(&form)
            .get_result::<Self>(conn.deref_mut());
        Ok(if is_conflict(&article) {
            update(article::table)
                .filter(article::ap_id.eq(form.ap_id.clone()))
                .set(form)
                .get_result::<Self>(conn.deref_mut())?
        } else {
            let a = article?;
            Notification::notify_article(&a, creator_id, context).await?;
            a
        })
    }

    pub fn update_text(id: ArticleId, text: &str, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set((article::dsl::text.eq(text), article::dsl::updated.eq(now)))
            .get_result(conn.deref_mut())?)
    }

    pub fn update_protected(
        id: ArticleId,
        locked: bool,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::protected.eq(locked))
            .get_result(conn.deref_mut())?)
    }

    pub fn update_removed(
        id: ArticleId,
        removed: bool,
        context: &IbisContext,
    ) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::removed.eq(removed))
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: ArticleId, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(article::table
            .find(id)
            .filter(not(article::removed))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_view<'a>(
        params: impl Into<ArticleViewQuery<'a>>,
        user: Option<&LocalUserView>,
        context: &IbisContext,
    ) -> BackendResult<ArticleView> {
        let mut conn = context.db_pool.get()?;
        let local_user_id = user.map(|u| u.local_user.id);
        let mut query = article::table
            .inner_join(instance::table)
            .left_join(
                article_follow::table.on(article_follow::article_id
                    .eq(article::id)
                    .and(article_follow::local_user_id.nullable().eq(local_user_id))),
            )
            .into_boxed();
        if !user.map(|u| u.local_user.admin).unwrap_or_default() {
            query = query.filter(not(article::removed));
        }
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

        let (article, instance, following): (Article, _, _) = query
            .select((
                article::all_columns,
                instance::all_columns,
                article_follow::local_user_id.nullable().is_not_null(),
            ))
            .get_result(conn.deref_mut())?;
        let comments = Comment::read_for_article(article.id, context)?;
        let latest_version = article.latest_edit_version(context)?;
        Ok(ArticleView {
            article,
            instance,
            comments,
            latest_version,
            following,
        })
    }

    pub fn read_from_ap_id(ap_id: &DbUrl, context: &IbisContext) -> BackendResult<Self> {
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
        include_removed: bool,
        context: &IbisContext,
    ) -> BackendResult<Vec<Self>> {
        let mut conn = context.db_pool.get()?;
        let mut query = article::table
            .inner_join(edit::table)
            .inner_join(instance::table)
            .group_by(article::id)
            .order_by(max(edit::published).desc())
            .select(article::all_columns)
            .into_boxed();

        if let Some(true) = only_local {
            query = query.filter(article::local);
        }
        if !include_removed {
            query = query
                .filter(article::removed.eq(false))
                .filter(article::pending.eq(false));
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
            .filter(not(article::removed))
            .filter(not(article::pending))
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
        follower: &LocalUserView,
        context: &IbisContext,
    ) -> BackendResult<()> {
        use article_follow::dsl::{article_id, local_user_id};
        let mut conn = context.db_pool.get()?;
        let form = (
            article_id.eq(article_id_),
            local_user_id.eq(follower.local_user.id),
        );
        insert_into(article_follow::table)
            .values(form)
            .execute(conn.deref_mut())?;
        Ok(())
    }

    pub fn unfollow(
        article_id_: ArticleId,
        follower: &LocalUserView,
        context: &IbisContext,
    ) -> BackendResult<()> {
        use article_follow::dsl::{article_id, local_user_id};
        let mut conn = context.db_pool.get()?;
        delete(
            article_follow::table.filter(
                article_id
                    .eq(article_id_)
                    .and(local_user_id.eq(follower.local_user.id)),
            ),
        )
        .execute(conn.deref_mut())?;
        Ok(())
    }
}
