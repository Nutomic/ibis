use crate::error::Error;
use crate::federation::objects::articles_collection::DbArticleCollection;
use crate::{database::DatabaseHandle, federation::activities::follow::Follow};
use activitypub_federation::activity_sending::SendActivityTask;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::kinds::actor::ServiceType;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::{context::WithContext, public_key::PublicKey, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbInstance {
    pub ap_id: ObjectId<DbInstance>,
    pub articles_id: CollectionId<DbArticleCollection>,
    pub inbox: Url,
    pub(crate) public_key: String,
    pub(crate) private_key: Option<String>,
    pub(crate) last_refreshed_at: DateTime<Utc>,
    pub followers: Vec<DbInstance>,
    pub follows: Vec<Url>,
    pub local: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    #[serde(rename = "type")]
    kind: ServiceType,
    id: ObjectId<DbInstance>,
    articles: CollectionId<DbArticleCollection>,
    inbox: Url,
    public_key: PublicKey,
}

impl DbInstance {
    pub fn followers_url(&self) -> Result<Url, Error> {
        Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
    }

    pub async fn follow(
        &self,
        other: &DbInstance,
        data: &Data<DatabaseHandle>,
    ) -> Result<(), Error> {
        let follow = Follow::new(self.ap_id.clone(), other.ap_id.clone())?;
        self.send(follow, vec![other.shared_inbox_or_inbox()], data)
            .await?;
        Ok(())
    }

    pub(crate) async fn send<Activity>(
        &self,
        activity: Activity,
        recipients: Vec<Url>,
        data: &Data<DatabaseHandle>,
    ) -> Result<(), <Activity as ActivityHandler>::Error>
    where
        Activity: ActivityHandler + Serialize + Debug + Send + Sync,
        <Activity as ActivityHandler>::Error: From<anyhow::Error> + From<serde_json::Error>,
    {
        let activity = WithContext::new_default(activity);
        let sends = SendActivityTask::prepare(&activity, self, recipients, data).await?;
        for send in sends {
            send.sign_and_send(data).await?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl Object for DbInstance {
    type DataType = DatabaseHandle;
    type Kind = Instance;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        Some(self.last_refreshed_at)
    }

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let users = data.instances.lock().unwrap();
        let res = users
            .clone()
            .into_iter()
            .map(|u| u.1)
            .find(|u| u.ap_id.inner() == &object_id);
        Ok(res)
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(Instance {
            kind: Default::default(),
            id: self.ap_id.clone(),
            articles: self.articles_id.clone(),
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
        let instance = DbInstance {
            ap_id: json.id,
            articles_id: json.articles,
            inbox: json.inbox,
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Local::now().into(),
            followers: vec![],
            follows: vec![],
            local: false,
        };
        // TODO: very inefficient to sync all articles every time
        instance.articles_id.dereference(&instance, data).await?;
        let mut mutex = data.instances.lock().unwrap();
        mutex.insert(instance.ap_id.inner().clone(), instance.clone());
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
        self.inbox.clone()
    }
}
