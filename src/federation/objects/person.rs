use crate::database::DatabaseHandle;
use crate::error::Error;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    http_signatures::generate_actor_keypair,
    kinds::actor::PersonType,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{Actor, Object},
};
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

#[derive(Debug, Clone)]
pub struct DbUser {
    pub name: String,
    pub ap_id: ObjectId<DbUser>,
    pub inbox: Url,
    public_key: String,
    private_key: Option<String>,
    last_refreshed_at: NaiveDateTime,
    pub followers: Vec<Url>,
    pub local: bool,
}

impl DbUser {
    pub fn new(hostname: &str, name: String) -> Result<DbUser, Error> {
        let ap_id = Url::parse(&format!("http://{}/{}", hostname, &name))?.into();
        let inbox = Url::parse(&format!("http://{}/{}/inbox", hostname, &name))?;
        let keypair = generate_actor_keypair()?;
        Ok(DbUser {
            name,
            ap_id,
            inbox,
            public_key: keypair.public_key,
            private_key: Some(keypair.private_key),
            last_refreshed_at: Local::now().naive_local(),
            followers: vec![],
            local: true,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(rename = "type")]
    kind: PersonType,
    preferred_username: String,
    id: ObjectId<DbUser>,
    inbox: Url,
    public_key: PublicKey,
}

#[async_trait::async_trait]
impl Object for DbUser {
    type DataType = DatabaseHandle;
    type Kind = Person;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
        Some(self.last_refreshed_at)
    }

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let users = data.users.lock().unwrap();
        let res = users
            .clone()
            .into_iter()
            .find(|u| u.ap_id.inner() == &object_id);
        Ok(res)
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(Person {
            preferred_username: self.name.clone(),
            kind: Default::default(),
            id: self.ap_id.clone(),
            inbox: self.inbox.clone(),
            public_key: self.public_key(),
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let user = DbUser {
            name: json.preferred_username,
            ap_id: json.id,
            inbox: json.inbox,
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Local::now().naive_local(),
            followers: vec![],
            local: false,
        };
        let mut mutex = data.users.lock().unwrap();
        mutex.push(user.clone());
        Ok(user)
    }
}

impl Actor for DbUser {
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
        self.inbox.clone()
    }
}
