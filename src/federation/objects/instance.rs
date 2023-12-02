use crate::database::instance::{DbInstance, DbInstanceForm};
use crate::database::MyDataHandle;
use crate::error::Error;
use crate::federation::objects::articles_collection::DbArticleCollection;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::kinds::actor::ServiceType;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::{public_key::PublicKey, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::{ParseError, Url};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubInstance {
    #[serde(rename = "type")]
    kind: ServiceType,
    id: ObjectId<DbInstance>,
    articles: CollectionId<DbArticleCollection>,
    inbox: Url,
    public_key: PublicKey,
}

#[async_trait::async_trait]
impl Object for DbInstance {
    type DataType = MyDataHandle;
    type Kind = ApubInstance;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(self.last_refreshed_at)
    }

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(DbInstance::read_from_ap_id(&object_id.into(), &data).ok())
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(ApubInstance {
            kind: Default::default(),
            id: self.ap_id.clone(),
            articles: self.articles_url.clone(),
            inbox: Url::parse(&self.inbox_url)?,
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
        let form = DbInstanceForm {
            ap_id: json.id,
            articles_url: json.articles,
            inbox_url: json.inbox.to_string(),
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Local::now().into(),
            local: false,
        };
        let instance = DbInstance::create(&form, &data.db_connection)?;
        // TODO: very inefficient to sync all articles every time
        instance.articles_url.dereference(&instance, data).await?;
        Ok(instance)
    }
}

impl Actor for DbInstance {
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
        Url::parse(&self.inbox_url).unwrap()
    }
}
