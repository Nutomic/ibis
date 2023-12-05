use crate::database::schema::user_;
use crate::database::MyDataHandle;
use crate::error::MyResult;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use chrono::{DateTime, Utc};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{
    insert_into, AsChangeset, Identifiable, Insertable, PgConnection, Queryable, RunQueryDsl,
    Selectable,
};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = user_, check_for_backend(diesel::pg::Pg))]
pub struct DbUser {
    pub id: i32,
    pub ap_id: ObjectId<DbUser>,
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
#[diesel(table_name = user_, check_for_backend(diesel::pg::Pg))]
pub struct DbUserForm {
    pub ap_id: ObjectId<DbUser>,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
}

impl DbUser {
    pub fn create(form: &DbUserForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(user_::table)
            .values(form)
            .on_conflict(user_::dsl::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbUser>,
        data: &Data<MyDataHandle>,
    ) -> MyResult<DbUser> {
        let mut conn = data.db_connection.lock().unwrap();
        Ok(user_::table
            .filter(user_::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }
}
