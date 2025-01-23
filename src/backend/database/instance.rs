use crate::{
    backend::{
        database::{
            schema::{article, comment, instance, instance_follow},
            IbisContext,
        },
        federation::objects::{
            articles_collection::DbArticleCollection,
            instance_collection::DbInstanceCollection,
        },
        utils::error::MyResult,
    },
    common::{
        instance::{DbInstance, InstanceView},
        newtypes::{CommentId, InstanceId},
        user::DbPerson,
    },
};
use activitypub_federation::{
    config::Data,
    fetch::{collection_id::CollectionId, object_id::ObjectId},
};
use chrono::{DateTime, Utc};
use diesel::{
    insert_into,
    update,
    AsChangeset,
    ExpressionMethods,
    Insertable,
    JoinOnDsl,
    QueryDsl,
    RunQueryDsl,
};
use std::{fmt::Debug, ops::DerefMut};

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstanceForm {
    pub domain: String,
    pub ap_id: ObjectId<DbInstance>,
    pub topic: Option<String>,
    pub articles_url: Option<CollectionId<DbArticleCollection>>,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
    pub instances_url: Option<CollectionId<DbInstanceCollection>>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstanceUpdateForm {
    pub topic: Option<String>,
    pub name: Option<String>,
}

impl DbInstance {
    pub fn create(form: &DbInstanceForm, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(insert_into(instance::table)
            .values(form)
            .on_conflict(instance::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: InstanceId, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn update(form: DbInstanceUpdateForm, context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(update(instance::table)
            .filter(instance::local)
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbInstance>,
        context: &Data<IbisContext>,
    ) -> MyResult<DbInstance> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table
            .filter(instance::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local(context: &IbisContext) -> MyResult<Self> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table
            .filter(instance::local.eq(true))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_view(
        id: Option<InstanceId>,
        context: &Data<IbisContext>,
    ) -> MyResult<InstanceView> {
        let instance = match id {
            Some(id) => DbInstance::read(id, context),
            None => DbInstance::read_local(context),
        }?;
        let followers = DbInstance::read_followers(instance.id, context)?;

        Ok(InstanceView {
            instance,
            followers,
        })
    }

    pub fn follow(
        follower: &DbPerson,
        instance: &DbInstance,
        pending_: bool,
        context: &Data<IbisContext>,
    ) -> MyResult<()> {
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
        assert_eq!(1, rows);
        Ok(())
    }

    pub fn read_followers(id_: InstanceId, context: &IbisContext) -> MyResult<Vec<DbPerson>> {
        use crate::backend::database::schema::person;
        use instance_follow::dsl::{follower_id, instance_id};
        let mut conn = context.db_pool.get()?;
        Ok(instance_follow::table
            .inner_join(person::table.on(follower_id.eq(person::id)))
            .filter(instance_id.eq(id_))
            .select(person::all_columns)
            .get_results(conn.deref_mut())?)
    }

    pub fn list(only_remote: bool, context: &Data<IbisContext>) -> MyResult<Vec<DbInstance>> {
        let mut conn = context.db_pool.get()?;
        let mut query = instance::table.into_boxed();
        if only_remote {
            query = query.filter(instance::local.eq(false));
        }
        Ok(query.get_results(conn.deref_mut())?)
    }

    /// Read the instance where an article is hosted, based on a comment id.
    /// Note this may be different from the instance where the comment is hosted.
    pub fn read_for_comment(
        comment_id: CommentId,
        context: &Data<IbisContext>,
    ) -> MyResult<DbInstance> {
        let mut conn = context.db_pool.get()?;
        Ok(instance::table
            .inner_join(article::table)
            .inner_join(comment::table.on(comment::article_id.eq(article::id)))
            .filter(comment::id.eq(comment_id))
            .select(instance::all_columns)
            .get_result(conn.deref_mut())?)
    }
}
