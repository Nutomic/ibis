use super::newtypes::{LocalUserId, PersonId};
use crate::DbUrl;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;
#[cfg(feature = "ssr")]
use {
    crate::schema::{local_user, person},
    diesel::{Identifiable, Queryable, Selectable},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct LocalUserView {
    pub person: Person,
    pub local_user: LocalUser,
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
    pub ap_id: DbUrl,
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
