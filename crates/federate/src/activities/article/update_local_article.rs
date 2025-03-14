use crate::{
    generate_activity_id,
    objects::{
        article::{ApubArticle, ArticleWrapper},
        instance::InstanceWrapper,
    },
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::UpdateType, public},
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use ibis_database::{
    common::instance::Instance,
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalArticle {
    pub actor: ObjectId<InstanceWrapper>,
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
        article: ArticleWrapper,
        extra_recipients: Vec<InstanceWrapper>,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        debug_assert!(article.local);
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        let id = generate_activity_id(context)?;
        let update = UpdateLocalArticle {
            actor: local_instance.ap_id.clone().into(),
            to: vec![public()],
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
        ArticleWrapper::from_json(self.object, context).await?;

        Ok(())
    }
}
