use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::actor::PersonType,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{Actor, Object},
};
use chrono::{DateTime, Utc};
use ibis_database::{
    common::user::Person,
    error::BackendError,
    impls::{IbisContext, user::PersonInsertForm},
};
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, ops::Deref};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubUser {
    #[serde(rename = "type")]
    kind: PersonType,
    id: ObjectId<PersonWrapper>,
    preferred_username: String,
    /// displayname
    name: Option<String>,
    summary: Option<String>,
    inbox: Url,
    public_key: PublicKey,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PersonWrapper(Person);

impl Deref for PersonWrapper {
    type Target = Person;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Person> for PersonWrapper {
    fn from(value: Person) -> Self {
        PersonWrapper(value)
    }
}

#[async_trait::async_trait]
impl Object for PersonWrapper {
    type DataType = IbisContext;
    type Kind = ApubUser;
    type Error = BackendError;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(self.last_refreshed_at)
    }

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(Person::read_from_ap_id(&object_id.into(), context)
            .ok()
            .map(Into::into))
    }

    async fn into_json(self, _context: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(ApubUser {
            kind: Default::default(),
            id: self.ap_id.clone().into(),
            preferred_username: self.username.clone(),
            inbox: Url::parse(&self.inbox_url)?,
            public_key: self.public_key(),
            name: self.display_name.clone(),
            summary: self.bio.clone(),
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
        let form = PersonInsertForm {
            username: json.preferred_username,
            ap_id: json.id.into(),
            inbox_url: json.inbox.to_string(),
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Utc::now(),
            local: false,
            display_name: json.name,
            bio: json.summary,
        };
        Person::create(&form, context).map(Into::into)
    }
}

impl Actor for PersonWrapper {
    fn id(&self) -> Url {
        self.ap_id.clone().into()
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
