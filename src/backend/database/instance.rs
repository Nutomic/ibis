use crate::backend::database::schema::{instance, instance_follow};
use crate::backend::database::MyDataHandle;
use crate::backend::error::MyResult;
use crate::backend::federation::objects::articles_collection::DbArticleCollection;
use crate::common::DbPerson;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use diesel::ExpressionMethods;
use diesel::{
    insert_into, AsChangeset, Identifiable, Insertable, JoinOnDsl, PgConnection, QueryDsl,
    Queryable, RunQueryDsl, Selectable,
};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::ops::DerefMut;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstance {
    pub id: i32,
    pub ap_id: ObjectId<DbInstance>,
    pub articles_url: CollectionId<DbArticleCollection>,
    pub inbox_url: String,
    #[serde(skip)]
    pub public_key: String,
    #[serde(skip)]
    pub private_key: Option<String>,
    #[serde(skip)]
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = instance, check_for_backend(diesel::pg::Pg))]
pub struct DbInstanceForm {
    pub ap_id: ObjectId<DbInstance>,
    pub articles_url: CollectionId<DbArticleCollection>,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct InstanceView {
    pub instance: DbInstance,
    pub followers: Vec<DbPerson>,
    pub following: Vec<DbInstance>,
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
        data: &Data<MyDataHandle>,
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

    pub fn read_local_view(conn: &Mutex<PgConnection>) -> MyResult<InstanceView> {
        let instance = DbInstance::read_local_instance(conn)?;
        let followers = DbInstance::read_followers(instance.id, conn)?;
        let following = DbInstance::read_following(instance.id, conn)?;

        Ok(InstanceView {
            instance,
            followers,
            following,
        })
    }

    pub fn follow(
        follower: &DbPerson,
        instance: &DbInstance,
        pending_: bool,
        data: &Data<MyDataHandle>,
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

    pub fn read_following(id_: i32, conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        use instance_follow::dsl::{follower_id, instance_id};
        let mut conn = conn.lock().unwrap();
        Ok(instance_follow::table
            .inner_join(instance::table.on(instance_id.eq(instance::dsl::id)))
            .filter(follower_id.eq(id_))
            .select(instance::all_columns)
            .get_results(conn.deref_mut())?)
    }
}
