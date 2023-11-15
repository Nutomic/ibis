use crate::error::Error;
use crate::{
    database::DatabaseHandle,
    federation::activities::{accept::Accept, follow::Follow},
};
use activitypub_federation::kinds::actor::ServiceType;
use activitypub_federation::{
    activity_queue::send_activity,
    config::Data,
    fetch::{object_id::ObjectId, webfinger::webfinger_resolve_actor},
    http_signatures::generate_actor_keypair,
    protocol::{context::WithContext, public_key::PublicKey, verification::verify_domains_match},
    traits::{ActivityHandler, Actor, Object},
};
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use url::Url;

#[derive(Debug, Clone)]
pub struct DbInstance {
    pub ap_id: ObjectId<DbInstance>,
    pub inbox: Url,
    public_key: String,
    private_key: Option<String>,
    last_refreshed_at: NaiveDateTime,
    pub followers: Vec<Url>,
    pub local: bool,
}

/// List of all activities which this actor can receive.
#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
#[enum_delegate::implement(ActivityHandler)]
pub enum PersonAcceptedActivities {
    Follow(Follow),
    Accept(Accept),
}

impl DbInstance {
    pub fn new(hostname: &str) -> Result<DbInstance, Error> {
        let ap_id = Url::parse(&format!("http://{}", hostname))?.into();
        let inbox = Url::parse(&format!("http://{}/inbox", hostname))?;
        let keypair = generate_actor_keypair()?;
        Ok(DbInstance {
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
pub struct Instance {
    #[serde(rename = "type")]
    kind: ServiceType,
    id: ObjectId<DbInstance>,
    inbox: Url,
    public_key: PublicKey,
}

impl DbInstance {
    pub fn followers(&self) -> &Vec<Url> {
        &self.followers
    }

    pub fn followers_url(&self) -> Result<Url, Error> {
        Ok(Url::parse(&format!("{}/followers", self.ap_id.inner()))?)
    }

    pub async fn follow(&self, other: &str, data: &Data<DatabaseHandle>) -> Result<(), Error> {
        let other: DbInstance = webfinger_resolve_actor(other, data).await?;
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
        send_activity(activity, self, recipients, data).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Object for DbInstance {
    type DataType = DatabaseHandle;
    type Kind = Instance;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<NaiveDateTime> {
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
            .find(|u| u.ap_id.inner() == &object_id);
        Ok(res)
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        Ok(Instance {
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
        let instance = DbInstance {
            ap_id: json.id,
            inbox: json.inbox,
            public_key: json.public_key.public_key_pem,
            private_key: None,
            last_refreshed_at: Local::now().naive_local(),
            followers: vec![],
            local: false,
        };
        let mut mutex = data.instances.lock().unwrap();
        mutex.push(instance.clone());
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
