use crate::database::schema::{local_user, person};
use crate::database::MyDataHandle;
use crate::error::MyResult;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::http_signatures::generate_actor_keypair;
use bcrypt::hash;
use bcrypt::DEFAULT_COST;
use chrono::{DateTime, Local, Utc};
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::{
    insert_into, AsChangeset, Identifiable, Insertable, PgConnection, Queryable, RunQueryDsl,
    Selectable,
};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct LocalUserView {
    pub person: DbPerson,
    pub local_user: DbLocalUser,
}

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
    pub fn create(person_form: &DbPersonForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(person::table)
            .values(person_form)
            .on_conflict(person::dsl::ap_id)
            .do_update()
            .set(person_form)
            .get_result::<DbPerson>(conn.deref_mut())?)
    }

    pub fn create_local(
        username: String,
        password: String,
        data: &Data<MyDataHandle>,
    ) -> MyResult<LocalUserView> {
        let mut conn = data.db_connection.lock().unwrap();
        let hostname = data.domain();
        let ap_id = ObjectId::parse(&format!("http://{hostname}/user/{username}"))?;
        let inbox_url = format!("http://{hostname}/inbox");
        let keypair = generate_actor_keypair()?;
        let person_form = DbPersonForm {
            username,
            ap_id,
            inbox_url,
            public_key: keypair.public_key,
            private_key: Some(keypair.private_key),
            last_refreshed_at: Local::now().into(),
            local: true,
        };

        let person = insert_into(person::table)
            .values(person_form)
            .get_result::<DbPerson>(conn.deref_mut())?;

        let local_user_form = DbLocalUserForm {
            password_encrypted: hash(password, DEFAULT_COST)?,
            person_id: person.id,
        };

        let local_user = insert_into(local_user::table)
            .values(local_user_form)
            .get_result::<DbLocalUser>(conn.deref_mut())?;

        Ok(LocalUserView { local_user, person })
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

    pub fn read_local_from_name(
        username: &str,
        data: &Data<MyDataHandle>,
    ) -> MyResult<LocalUserView> {
        let mut conn = data.db_connection.lock().unwrap();
        Ok(person::table
            .inner_join(local_user::table)
            .filter(person::dsl::local)
            .filter(person::dsl::username.eq(username))
            .get_result(conn.deref_mut())?)
    }
}
