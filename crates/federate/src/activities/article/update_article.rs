use crate::{
    activities::announce::AnnounceActivity,
    generate_activity_id,
    objects::{
        article::{ApubArticle, ArticleWrapper},
        instance::InstanceWrapper,
    },
    routes::AnnouncableActivities,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::UpdateType, public},
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use ibis_database::{
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
    pub actor: ObjectId<InstanceWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubArticle,
    #[serde(rename = "type")]
    pub kind: UpdateType,
    pub id: Url,
}

impl UpdateArticle {
    /// Sent from article origin instance
    pub async fn send(
        article: ArticleWrapper,
        local_instance: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let id = generate_activity_id(context)?;
        let update = UpdateArticle {
            actor: local_instance.ap_id.clone().into(),
            to: vec![local_instance.ap_id.clone().into(), public()],
            object: article.into_json(context).await?,
            kind: Default::default(),
            id,
        };
        AnnounceActivity::send(AnnouncableActivities::UpdateArticle(update), context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UpdateArticle {
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

    /// Ignored by Ibis, this is for other platforms
    async fn receive(self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }
}
