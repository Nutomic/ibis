use crate::backend::database::schema::{instance, instance_follow};
use crate::backend::database::IbisData;
use crate::backend::error::MyResult;
use crate::backend::federation::objects::articles_collection::DbArticleCollection;
use crate::common::{DbInstance, DbPerson, InstanceView};
use activitypub_federation::config::Data;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use diesel::ExpressionMethods;
use diesel::{
    insert_into, AsChangeset, Insertable, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
};
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::Mutex;

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstanceForm {
    pub ap_id: ObjectId<DbInstance>,
    pub description: Option<String>,
    pub articles_url: CollectionId<DbArticleCollection>,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
}

impl DbInstance {
    pub fn create(form: &DbInstanceForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(instance::table)
            .values(form)
            .on_conflict(instance::dsl::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read(id: i32, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(instance::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbInstance>,
        data: &Data<IbisData>,
    ) -> MyResult<DbInstance> {
        let mut conn = data.db_connection.lock().unwrap();
        Ok(instance::table
            .filter(instance::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local_instance(conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(instance::table
            .filter(instance::dsl::local.eq(true))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local_view(data: &Data<IbisData>) -> MyResult<InstanceView> {
        let instance = DbInstance::read_local_instance(&data.db_connection)?;
        let followers = DbInstance::read_followers(instance.id, &data.db_connection)?;

        Ok(InstanceView {
            instance,
            followers,
            registration_open: data.config.registration_open,
        })
    }

    pub fn follow(
        follower: &DbPerson,
        instance: &DbInstance,
        pending_: bool,
        data: &Data<IbisData>,
    ) -> MyResult<()> {
        use instance_follow::dsl::{follower_id, instance_id, pending};
        let mut conn = data.db_connection.lock().unwrap();
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

    pub fn read_followers(id_: i32, conn: &Mutex<PgConnection>) -> MyResult<Vec<DbPerson>> {
        use crate::backend::database::schema::person;
        use instance_follow::dsl::{follower_id, instance_id};
        let mut conn = conn.lock().unwrap();
        Ok(instance_follow::table
            .inner_join(person::table.on(follower_id.eq(person::dsl::id)))
            .filter(instance_id.eq(id_))
            .select(person::all_columns)
            .get_results(conn.deref_mut())?)
    }
}
