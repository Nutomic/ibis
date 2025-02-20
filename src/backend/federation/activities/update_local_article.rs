use crate::{
    backend::{
        database::IbisContext,
        federation::objects::article::ApubArticle,
        utils::{
            error::{BackendError, BackendResult},
            generate_activity_id,
        },
    },
    common::{article::Article, instance::Instance},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UpdateType,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalArticle {
    pub actor: ObjectId<Instance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubArticle,
    #[serde(rename = "type")]
    pub kind: UpdateType,
    pub id: Url,
}

impl UpdateLocalArticle {
    /// Sent from article origin instance
    pub async fn send(
        article: Article,
        extra_recipients: Vec<Instance>,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        debug_assert!(article.local);
        let local_instance = Instance::read_local(context)?;
        let id = generate_activity_id(context)?;
        let mut to = local_instance.follower_ids(context)?;
        to.extend(extra_recipients.iter().map(|i| i.ap_id.inner().clone()));
        let update = UpdateLocalArticle {
            actor: local_instance.ap_id.clone(),
            to,
            object: article.into_json(context).await?,
            kind: Default::default(),
            id,
        };
        local_instance
            .send_to_followers(update, extra_recipients, context)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UpdateLocalArticle {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Received on article follower instances (where article is always remote)
    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Article::from_json(self.object, context).await?;

        Ok(())
    }
}
