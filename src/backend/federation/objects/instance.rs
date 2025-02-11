use super::instance_collection::DbInstanceCollection;
use crate::{
    backend::{
        database::{instance::DbInstanceForm, IbisContext},
        federation::{objects::articles_collection::DbArticleCollection, send_activity},
        utils::error::{BackendError, BackendResult},
    },
    common::{instance::DbInstance, utils::extract_domain},
};
use activitypub_federation::{
    config::Data,
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    kinds::actor::ServiceType,
    protocol::{
        public_key::PublicKey,
        verification::{verify_domains_match, verify_is_remote_object},
    },
    traits::{ActivityHandler, Actor, Object},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubInstance {
    #[serde(rename = "type")]
    kind: ServiceType,
    pub id: ObjectId<DbInstance>,
    name: Option<String>,
    summary: Option<String>,
    articles: Option<CollectionId<DbArticleCollection>>,
    instances: Option<CollectionId<DbInstanceCollection>>,
    inbox: Url,
    public_key: PublicKey,
}

impl DbInstance {
    pub fn followers_url(&self) -> BackendResult<Url> {
        Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
    }

    pub fn follower_ids(&self, context: &Data<IbisContext>) -> BackendResult<Vec<Url>> {
        Ok(DbInstance::read_followers(self.id, context)?
            .into_iter()
            .map(|f| f.ap_id.into())
            .collect())
    }

    pub async fn send_to_followers<Activity>(
        &self,
        activity: Activity,
        extra_recipients: Vec<DbInstance>,
        context: &Data<IbisContext>,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Debug + Send + Sync,
        <Activity as ActivityHandler>::Error: From<activitypub_federation::error::Error>,
        <Activity as ActivityHandler>::Error: From<BackendError>,
    {
        let mut inboxes: Vec<_> = DbInstance::read_followers(self.id, context)?
            .iter()
            .map(|f| f.inbox_url())
            .collect();
        inboxes.extend(extra_recipients.into_iter().map(|i| i.inbox_url()));
        send_activity(self, activity, inboxes, context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Object for DbInstance {
    type DataType = IbisContext;
    type Kind = ApubInstance;
    type Error = BackendError;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(self.last_refreshed_at)
    }

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(DbInstance::read_from_ap_id(&object_id.into(), context).ok())
    }

    async fn into_json(self, _context: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(ApubInstance {
            kind: Default::default(),
            id: self.ap_id.clone(),
            summary: self.topic.clone(),
            articles: self.articles_url.clone(),
            instances: self.instances_url.clone(),
            inbox: Url::parse(&self.inbox_url)?,
            public_key: self.public_key(),
            name: self.name,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        context: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        verify_is_remote_object(&json.id, context)?;
        Ok(())
    }

    async fn from_json(
        json: Self::Kind,
        context: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let domain = extract_domain(&json.id);
        let form = DbInstanceForm {
            domain,
            ap_id: json.id,
            topic: json.summary,
            articles_url: json.articles,
            instances_url: json.instances,
            inbox_url: json.inbox.to_string(),
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Utc::now(),
            local: false,
            name: json.name,
        };
        let instance = DbInstance::create(&form, context)?;

        // TODO: very inefficient to sync all articles every time
        let instance_ = instance.clone();
        let context_ = context.reset_request_count();
        tokio::spawn(async move {
            if let Some(articles_url) = &instance_.articles_url {
                let res = articles_url.dereference(&(), &context_).await;
                if let Err(e) = res {
                    tracing::warn!("error in spawn: {e}");
                }
            }
            if let Some(instances_url) = &instance_.instances_url {
                let res = instances_url.dereference(&(), &context_).await;
                if let Err(e) = res {
                    tracing::warn!("error in spawn: {e}");
                }
            }
        });

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
        self.inbox_url()
    }
}
