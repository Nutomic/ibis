use crate::{
    backend::{
        database::IbisContext,
        federation::objects::article::ApubArticle,
        utils::{
            error::{Error, MyResult},
            generate_activity_id,
        },
    },
    common::{article::DbArticle, instance::DbInstance},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::CreateType,
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
        context: &Data<IbisContext>,
    ) -> MyResult<()> {
        let local_instance = DbInstance::read_local(context)?;
        let object = article.clone().into_json(context).await?;
        let id = generate_activity_id(context)?;
        let to = local_instance.follower_ids(context)?;
        let create = CreateArticle {
            actor: local_instance.ap_id.clone(),
            to,
            object,
            kind: Default::default(),
            id,
        };
        local_instance
            .send_to_followers(create, vec![], context)
            .await?;
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for CreateArticle {
    type DataType = IbisContext;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = DbArticle::from_json(self.object.clone(), context).await?;
        if article.local {
            let local_instance = DbInstance::read_local(context)?;
            local_instance
                .send_to_followers(self, vec![], context)
                .await?;
        }
        Ok(())
    }
}
