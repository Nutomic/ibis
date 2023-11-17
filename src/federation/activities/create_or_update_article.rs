use crate::database::DatabaseHandle;
use crate::error::MyResult;
use crate::federation::objects::article::{Article, DbArticle};
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_object_id;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum CreateOrUpdateType {
    Create,
    Update,
}

// TODO: temporary placeholder, later rework this to send diffs
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateArticle {
    pub actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: Article,
    #[serde(rename = "type")]
    pub kind: CreateOrUpdateType,
    pub id: Url,
}

impl CreateOrUpdateArticle {
    pub async fn send_to_local_followers(
        article: DbArticle,
        kind: CreateOrUpdateType,
        data: &Data<DatabaseHandle>,
    ) -> MyResult<()> {
        let local_instance = data.local_instance();
        let to = local_instance
            .followers
            .iter()
            .map(|f| f.ap_id.inner().clone())
            .collect();
        let object = article.clone().into_json(data).await?;
        let id = generate_object_id(local_instance.ap_id.inner())?;
        let create_or_update = CreateOrUpdateArticle {
            actor: local_instance.ap_id.clone(),
            to,
            object,
            kind,
            id,
        };
        let inboxes = local_instance
            .followers
            .iter()
            .map(|f| f.inbox.clone())
            .collect();
        local_instance.send(create_or_update, inboxes, data).await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdateArticle {
    type DataType = DatabaseHandle;
    type Error = crate::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        DbArticle::from_json(self.object, data).await?;
        Ok(())
    }
}
