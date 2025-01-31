use crate::{
    backend::{
        database::IbisContext,
        federation::{routes::AnnouncableActivities, send_activity},
        utils::{
            error::{BackendError, BackendResult},
            generate_activity_id,
        },
    },
    common::instance::DbInstance,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{activity::AnnounceType, public},
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Actor},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnnounceActivity {
    pub(crate) actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: AnnouncableActivities,
    #[serde(rename = "type")]
    pub(crate) kind: AnnounceType,
    pub(crate) id: Url,
}

impl AnnounceActivity {
    pub async fn send(
        object: AnnouncableActivities,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let id = generate_activity_id(context)?;
        let instance = DbInstance::read_local(context)?;
        let announce = AnnounceActivity {
            actor: instance.id().into(),
            to: vec![public()],
            object,
            kind: AnnounceType::Announce,
            id,
        };

        // Send to followers of instance
        let follower_inboxes = DbInstance::read_followers(instance.id, context)?
            .into_iter()
            .map(|f| f.inbox_url())
            .collect();
        send_activity(&instance, announce, follower_inboxes, context).await?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for AnnounceActivity {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    #[tracing::instrument(skip_all)]
    async fn verify(&self, _context: &Data<Self::DataType>) -> BackendResult<()> {
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn receive(self, context: &Data<Self::DataType>) -> BackendResult<()> {
        self.object.verify(context).await?;
        self.object.receive(context).await
    }
}
