use crate::{
    DbUrl,
    common::{
        instance::{Instance, InstanceView, InstanceWithArticles},
        newtypes::{CommentId, InstanceId},
        user::Person,
    },
    error::BackendResult,
    impls::IbisContext,
    schema::{article, comment, edit, instance, instance_follow},
};
use chrono::{DateTime, Utc};
use diesel::{
    dsl::{max, not},
    *,
};
use std::{fmt::Debug, ops::DerefMut};

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstanceForm {
    pub domain: String,
    pub ap_id: DbUrl,
    pub topic: Option<String>,
    pub articles_url: Option<DbUrl>,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
    pub instances_url: Option<DbUrl>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstanceUpdateForm {
    pub topic: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug)]
pub enum InstanceViewQuery<'a> {
    Id(InstanceId),
    Hostname(&'a str),
}

impl Instance {
    pub fn create(form: &DbInstanceForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(instance::table)
            .values(form)
            .on_conflict(instance::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: InstanceId, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn update(form: DbInstanceUpdateForm, context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(update(instance::table)
            .filter(instance::local)
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(ap_id: &DbUrl, context: &IbisContext) -> BackendResult<Instance> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table
            .filter(instance::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local(context: &IbisContext) -> BackendResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table
            .filter(instance::local.eq(true))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_view(
        params: InstanceViewQuery,
        context: &IbisContext,
    ) -> BackendResult<InstanceView> {
        let mut conn = context.db_pool.get()?;
        let query = match params {
            InstanceViewQuery::Id(id) => instance::table.find(id).into_boxed(),
            InstanceViewQuery::Hostname(hostname) => instance::table
                .filter(instance::domain.eq(hostname))
                .into_boxed(),
        };
        let (instance, following) = query
            .left_join(instance_follow::table)
            .select((
                instance::all_columns,
                not(instance_follow::pending).nullable(),
            ))
            .get_result::<(Instance, Option<bool>)>(conn.deref_mut())?;

        Ok(InstanceView {
            instance,
            following: following.unwrap_or_default(),
        })
    }

    pub fn follow(
        follower: &Person,
        instance: &Instance,
        pending_: bool,
        context: &IbisContext,
    ) -> BackendResult<()> {
        use instance_follow::dsl::{follower_id, instance_id, pending};
        let mut conn = context.db_pool.get()?;
        let form = (
            instance_id.eq(instance.id),
            follower_id.eq(follower.id),
            pending.eq(pending_),
        );
        let rows = insert_into(instance_follow::table)
            .values(form)
            .on_conflict((instance_id, follower_id))
            .do_update()
            .set(form)
            .execute(conn.deref_mut())?;
        debug_assert_eq!(1, rows);
        Ok(())
    }

    pub fn unfollow(
        follower: &Person,
        instance: &Instance,
        context: &IbisContext,
    ) -> BackendResult<()> {
        use instance_follow::dsl::{follower_id, instance_id};
        let mut conn = context.db_pool.get()?;
        let rows = delete(instance_follow::table)
            .filter(instance_id.eq(instance.id).and(follower_id.eq(follower.id)))
            .execute(conn.deref_mut())?;
        debug_assert_eq!(1, rows);
        Ok(())
    }

    pub fn read_followers(id_: InstanceId, context: &IbisContext) -> BackendResult<Vec<Person>> {
        use crate::schema::person;
        use instance_follow::dsl::{follower_id, instance_id};
        let mut conn = context.db_pool.get()?;
        Ok(instance_follow::table
            .inner_join(person::table.on(follower_id.eq(person::id)))
            .filter(instance_id.eq(id_))
            .select(person::all_columns)
            .get_results(conn.deref_mut())?)
    }

    pub fn list(context: &IbisContext) -> BackendResult<Vec<Instance>> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table
            .filter(instance::local.eq(false))
            .get_results(conn.deref_mut())?)
    }

    pub fn list_with_articles(context: &IbisContext) -> BackendResult<Vec<InstanceWithArticles>> {
        let mut conn = context.db_pool.get()?;
        // select all instances, with most recently edited first (pending edits are ignored)
        let instances = instance::table
            // need to join manually, otherwise the order is wrong
            .left_join(article::table.on(article::instance_id.eq(instance::id)))
            .left_join(edit::table.on(article::id.eq(edit::article_id).and(not(edit::pending))))
            .group_by(instance::id)
            .order_by(max(edit::published).desc())
            .select(instance::all_columns)
            .get_results::<Instance>(conn.deref_mut())?;
        let mut res = vec![];
        // Get the last edited articles for each instance.
        // TODO: This is very inefficient, should use single query with lateral join
        // https://github.com/diesel-rs/diesel/discussions/4450
        for instance in instances {
            let articles = article::table
                .filter(article::instance_id.eq(instance.id))
                .inner_join(edit::table)
                .group_by(article::id)
                .order_by((article::local.desc(), max(edit::published).desc()))
                .limit(5)
                .select(article::all_columns)
                .get_results(conn.deref_mut())?;
            res.push(InstanceWithArticles { instance, articles });
        }

        Ok(res)
    }

    /// Read the instance where an article is hosted, based on a comment id.
    /// Note this may be different from the instance where the comment is hosted.
    pub fn read_for_comment(
        comment_id: CommentId,
        context: &IbisContext,
    ) -> BackendResult<Instance> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table
            .inner_join(article::table)
            .inner_join(comment::table.on(comment::article_id.eq(article::id)))
            .filter(comment::id.eq(comment_id))
            .select(instance::all_columns)
            .get_result(conn.deref_mut())?)
    }
}
