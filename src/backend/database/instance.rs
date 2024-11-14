use crate::{
    backend::{
        database::{
            schema::{instance, instance_follow},
            IbisData,
        },
        error::MyResult,
        federation::objects::{
            articles_collection::DbArticleCollection,
            instance_collection::DbInstanceCollection,
        },
    },
    common::{newtypes::InstanceId, DbInstance, DbPerson, InstanceView},
};
use activitypub_federation::{
    config::Data,
    fetch::{collection_id::CollectionId, object_id::ObjectId},
};
use chrono::{DateTime, Utc};
use diesel::{
    insert_into,
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
    pub description: Option<String>,
    pub articles_url: Option<CollectionId<DbArticleCollection>>,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
    pub instances_url: Option<CollectionId<DbInstanceCollection>>,
}

impl DbInstance {
    pub fn create(form: &DbInstanceForm, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(insert_into(instance::table)
            .values(form)
            .on_conflict(instance::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: InstanceId, data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(instance::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbInstance>,
        data: &Data<IbisData>,
    ) -> MyResult<DbInstance> {
        let mut conn = data.db_pool.get()?;
        Ok(instance::table
            .filter(instance::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local_instance(data: &IbisData) -> MyResult<Self> {
        let mut conn = data.db_pool.get()?;
        Ok(instance::table
            .filter(instance::local.eq(true))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_view(id: Option<InstanceId>, data: &Data<IbisData>) -> MyResult<InstanceView> {
        let instance = match id {
            Some(id) => DbInstance::read(id, data),
            None => DbInstance::read_local_instance(data),
        }?;
        let followers = DbInstance::read_followers(instance.id, data)?;

        Ok(InstanceView {
            instance,
            followers,
        })
    }

    pub fn follow(
        follower: &DbPerson,
        instance: &DbInstance,
        pending_: bool,
        data: &Data<IbisData>,
    ) -> MyResult<()> {
        use instance_follow::dsl::{follower_id, instance_id, pending};
        let mut conn = data.db_pool.get()?;
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

    pub fn read_followers(id_: InstanceId, data: &IbisData) -> MyResult<Vec<DbPerson>> {
        use crate::backend::database::schema::person;
        use instance_follow::dsl::{follower_id, instance_id};
        let mut conn = data.db_pool.get()?;
        Ok(instance_follow::table
            .inner_join(person::table.on(follower_id.eq(person::id)))
            .filter(instance_id.eq(id_))
            .select(person::all_columns)
            .get_results(conn.deref_mut())?)
    }

    pub fn read_remote(data: &Data<IbisData>) -> MyResult<Vec<DbInstance>> {
        let mut conn = data.db_pool.get()?;
        Ok(instance::table
            .filter(instance::local.eq(false))
            .get_results(conn.deref_mut())?)
    }
}
