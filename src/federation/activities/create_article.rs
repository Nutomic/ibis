use crate::database::DatabaseHandle;
use crate::error::MyResult;
use crate::federation::objects::article::{ApubArticle, DbArticle};
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_activity_id;
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
    pub async fn send_to_followers(
        article: DbArticle,
        data: &Data<DatabaseHandle>,
    ) -> MyResult<()> {
        let local_instance = data.local_instance();
        let object = article.clone().into_json(data).await?;
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let create_or_update = CreateArticle {
            actor: local_instance.ap_id.clone(),
            to: local_instance.follower_ids(),
            object,
            kind: Default::default(),
            id,
        };
        local_instance
            .send_to_followers(create_or_update, data)
            .await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for CreateArticle {
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
