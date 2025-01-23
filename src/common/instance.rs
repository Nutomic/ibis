use super::{
    newtypes::InstanceId,
    user::{DbPerson, LocalUserView},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use url::Url;
#[cfg(feature = "ssr")]
use {
    crate::backend::{
        database::schema::instance,
        federation::objects::articles_collection::DbArticleCollection,
        federation::objects::instance_collection::DbInstanceCollection,
    },
    activitypub_federation::fetch::{collection_id::CollectionId, object_id::ObjectId},
    diesel::{Identifiable, Queryable, Selectable},
    doku::Document,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "ssr", diesel(table_name = instance, check_for_backend(diesel::pg::Pg)))]
pub struct DbInstance {
    pub id: InstanceId,
    pub domain: String,
    #[cfg(feature = "ssr")]
    pub ap_id: ObjectId<DbInstance>,
    #[cfg(not(feature = "ssr"))]
    pub ap_id: String,
    pub topic: Option<String>,
    #[cfg(feature = "ssr")]
    pub articles_url: Option<CollectionId<DbArticleCollection>>,
    #[cfg(not(feature = "ssr"))]
    pub articles_url: String,
    pub inbox_url: String,
    #[serde(skip)]
    pub public_key: String,
    #[serde(skip)]
    pub private_key: Option<String>,
    pub last_refreshed_at: DateTime<Utc>,
    pub local: bool,
    #[cfg(feature = "ssr")]
    pub instances_url: Option<CollectionId<DbInstanceCollection>>,
    pub name: Option<String>,
}

impl DbInstance {
    pub fn inbox_url(&self) -> Url {
        Url::parse(&self.inbox_url).expect("can parse inbox url")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(table_name = article, check_for_backend(diesel::pg::Pg)))]
pub struct InstanceView {
    pub instance: DbInstance,
    pub followers: Vec<DbPerson>,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct SiteView {
    pub my_profile: Option<LocalUserView>,
    pub config: Options,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct GetInstanceParams {
    pub id: Option<InstanceId>,
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
