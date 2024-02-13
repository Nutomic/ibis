use crate::backend::database::IbisData;
use crate::backend::error::MyResult;
use crate::backend::federation::objects::article::ApubArticle;
use crate::backend::utils::generate_activity_id;
use crate::common::DbArticle;
use crate::common::DbInstance;
use activitypub_federation::kinds::activity::CreateType;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateArticle {
    pub actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubArticle,
    #[serde(rename = "type")]
    pub kind: CreateType,
    pub id: Url,
}

impl CreateArticle {
    pub async fn send_to_followers(article: DbArticle, data: &Data<IbisData>) -> MyResult<()> {
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        let object = article.clone().into_json(data).await?;
        let id = generate_activity_id(&local_instance.ap_id)?;
        let to = local_instance.follower_ids(data)?;
        let create = CreateArticle {
            actor: local_instance.ap_id.clone(),
            to,
            object,
            kind: Default::default(),
            id,
        };
        local_instance
            .send_to_followers(create, vec![], data)
            .await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for CreateArticle {
    type DataType = IbisData;
    type Error = crate::backend::error::Error;

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
        let article = DbArticle::from_json(self.object.clone(), data).await?;
        if article.local {
            let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
            local_instance.send_to_followers(self, vec![], data).await?;
        }
        Ok(())
    }
}
