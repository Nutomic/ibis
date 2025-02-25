use super::{
    instance::Instance,
    newtypes::{LocalUserId, PersonId},
};
use crate::{
    schema::{local_user, person},
    DbUrl,
};
use chrono::{DateTime, Utc};
#[cfg(feature = "ssr")]
use diesel::{Identifiable, Queryable, Selectable};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct RegisterUserParams {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct LoginUserParams {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct LocalUserView {
    pub person: Person,
    pub local_user: LocalUser,
    pub following: Vec<Instance>,
}

/// A user with account registered on local instance.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "ssr", diesel(table_name = local_user, check_for_backend(diesel::pg::Pg)))]
pub struct LocalUser {
    pub id: LocalUserId,
    #[serde(skip)]
    pub password_encrypted: String,
    pub person_id: PersonId,
    pub admin: bool,
}

/// Federation related data from a local or remote user.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "ssr", diesel(table_name = person, check_for_backend(diesel::pg::Pg)))]
pub struct Person {
    pub id: PersonId,
    pub username: String,
    #[cfg(feature = "ssr")]
    pub ap_id: DbUrl,
    #[cfg(not(feature = "ssr"))]
    pub ap_id: String,
    pub inbox_url: String,
    #[serde(skip)]
    pub public_key: String,
    #[serde(skip)]
    pub private_key: Option<String>,
    #[serde(skip)]
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
    pub display_name: Option<String>,
    pub bio: Option<String>,
}

impl Person {
    pub fn inbox_url(&self) -> Url {
        Url::parse(&self.inbox_url).expect("can parse inbox url")
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GetUserParams {
    pub name: String,
    pub domain: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateUserParams {
    pub person_id: PersonId,
    pub display_name: Option<String>,
    pub bio: Option<String>,
}
