use super::follow::Follow;
use crate::{
    generate_activity_id,
    objects::{instance::InstanceWrapper, user::PersonWrapper},
    send_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UndoType,
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
};
use ibis_database::{
    common::instance::Instance,
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoFollow {
    pub actor: ObjectId<PersonWrapper>,
    pub(crate) object: Follow,
    #[serde(rename = "type")]
    pub(crate) kind: UndoType,
    pub(crate) id: Url,
}

impl UndoFollow {
    pub async fn send(
        actor: &PersonWrapper,
        to: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let id = generate_activity_id(context)?;
        let undo_follow = UndoFollow {
            actor: actor.ap_id.clone().into(),
            object: Follow::new(actor, to, context)?,
            kind: Default::default(),
            id,
        };
        send_activity(
            actor,
            undo_follow,
            vec![to.shared_inbox_or_inbox()],
            context,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoFollow {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
        self.object.verify(context).await?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let actor = self.actor.dereference(context).await?;
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        verify_urls_match(self.object.object.inner(), local_instance.ap_id.inner())?;
        Instance::unfollow(&actor, &local_instance, context)?;

        Ok(())
    }
}
