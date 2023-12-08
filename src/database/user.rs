use crate::database::schema::{local_user, person};
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

/// A user with account registered on local instance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = local_user, check_for_backend(diesel::pg::Pg))]
pub struct DbLocalUser {
    pub id: i32,
    pub password_encrypted: String,
    pub person_id: i32,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = local_user, check_for_backend(diesel::pg::Pg))]
pub struct DbLocalUserForm {
    pub password_encrypted: String,
    pub person_id: i32,
}

/// Federation related data from a local or remote user.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = person, check_for_backend(diesel::pg::Pg))]
pub struct DbPerson {
    pub id: i32,
    pub username: String,
    pub ap_id: ObjectId<DbPerson>,
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
#[diesel(table_name = person, check_for_backend(diesel::pg::Pg))]
pub struct DbPersonForm {
    pub username: String,
    pub ap_id: ObjectId<DbPerson>,
    pub inbox_url: String,
    pub public_key: String,
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
}

impl DbPerson {
    pub fn create(person_form: &DbPersonForm, local_user_form: Option<&DbLocalUserForm>, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        let person = insert_into(person::table)
            .values(person_form)
            .on_conflict(person::dsl::ap_id)
            .do_update()
            .set(person_form)
            .get_result::<DbPerson>(conn.deref_mut())?;

        if let Some(local_user_form) = local_user_form {
            insert_into(local_user::table)
                .values(local_user_form)
                .get_result::<DbLocalUser>(conn.deref_mut())?;
        }

        Ok(person)
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbPerson>,
        data: &Data<MyDataHandle>,
    ) -> MyResult<DbPerson> {
        let mut conn = data.db_connection.lock().unwrap();
        Ok(person::table
            .filter(person::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }
}
