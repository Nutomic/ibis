use super::{
    article::Article,
    newtypes::InstanceId,
    user::{LocalUserView, Person},
};
use crate::DbUrl;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use url::Url;
#[cfg(feature = "ssr")]
use {
    crate::schema::instance,
    diesel::{Identifiable, Queryable, Selectable},
    doku::Document,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "ssr", diesel(table_name = instance, check_for_backend(diesel::pg::Pg)))]
pub struct Instance {
    pub id: InstanceId,
    pub domain: String,
    pub ap_id: DbUrl,
    pub topic: Option<String>,
    pub articles_url: Option<DbUrl>,
    pub inbox_url: String,
    #[serde(skip)]
    pub public_key: String,
    #[serde(skip)]
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
    #[cfg(feature = "ssr")]
    pub instances_url: Option<DbUrl>,
    pub name: Option<String>,
}

impl Instance {
    pub fn inbox_url(&self) -> Url {
        Url::parse(&self.inbox_url).expect("can parse inbox url")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InstanceView {
    pub instance: Instance,
    pub articles: Vec<Article>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(table_name = article, check_for_backend(diesel::pg::Pg)))]
pub struct InstanceView2 {
    pub instance: Instance,
    pub followers: Vec<Person>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, SmartDefault)]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "ssr", derive(Queryable, Document))]
#[cfg_attr(feature = "ssr", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct Options {
    /// Whether users can create new accounts
    #[default = true]
    #[cfg_attr(feature = "ssr", doku(example = "true"))]
    pub registration_open: bool,
    /// Whether admins need to approve new articles
    #[default = false]
    #[cfg_attr(feature = "ssr", doku(example = "false"))]
    pub article_approval: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct SiteView {
    pub my_profile: Option<LocalUserView>,
    pub config: Options,
    pub admin: Person,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetInstanceParams {
    pub id: Option<InstanceId>,
    pub hostname: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct FollowInstanceParams {
    pub id: InstanceId,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct UpdateInstanceParams {
    pub name: Option<String>,
    pub topic: Option<String>,
}
