use crate::{
    backend::{
        database::{user::DbPersonForm, IbisContext},
        utils::error::Error,
    },
    common::user::DbPerson,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::actor::PersonType,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubUser {
    #[serde(rename = "type")]
    kind: PersonType,
    id: ObjectId<DbPerson>,
    preferred_username: String,
    /// displayname
    name: Option<String>,
    summary: Option<String>,
    inbox: Url,
    public_key: PublicKey,
}

#[async_trait::async_trait]
impl Object for DbPerson {
    type DataType = IbisContext;
    type Kind = ApubUser;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(self.last_refreshed_at)
    }

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(DbPerson::read_from_ap_id(&object_id.into(), context).ok())
    }

    async fn into_json(self, _context: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(ApubUser {
            kind: Default::default(),
            id: __self.ap_id.clone(),
            preferred_username: __self.username.clone(),
            inbox: Url::parse(&__self.inbox_url)?,
            public_key: __self.public_key(),
            name: self.display_name,
            summary: self.bio,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _context: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        Ok(())
    }

    async fn from_json(
        json: Self::Kind,
        context: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let form = DbPersonForm {
            username: json.preferred_username,
            ap_id: json.id,
            inbox_url: json.inbox.to_string(),
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Utc::now(),
            local: false,
            display_name: json.name,
            bio: json.summary,
        };
        DbPerson::create(&form, context)
    }
}

impl Actor for DbPerson {
    fn id(&self) -> Url {
        self.ap_id.inner().clone()
    }

    fn public_key_pem(&self) -> &str {
        &self.public_key
    }

    fn private_key_pem(&self) -> Option<String> {
        self.private_key.clone()
    }

    fn inbox(&self) -> Url {
        self.inbox_url()
    }
}
