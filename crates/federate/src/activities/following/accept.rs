use super::InstanceOrPerson;
use crate::{
    activities::following::follow::Follow,
    generate_activity_id,
    objects::user::PersonWrapper,
    send_ibis_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::AcceptType,
    protocol::helpers::deserialize_skip_error,
    traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use either::Either;
use ibis_database::{
    common::instance::Instance,
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
    actor: ObjectId<InstanceOrPerson>,
    /// Optional, for compatibility with platforms that always expect recipient field
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) to: Option<[ObjectId<PersonWrapper>; 1]>,
    object: Follow,
    #[serde(rename = "type")]
    kind: AcceptType,
    id: Url,
}

impl Accept {
    pub async fn send(
        actor: InstanceOrPerson,
        object: Follow,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let id = generate_activity_id(context)?;
        let follower = object.actor.dereference(context).await?;
        let actor_id = match &actor {
            Either::Left(i) => i.ap_id.clone().into(),
            Either::Right(p) => p.ap_id.clone().into(),
        };
        let accept = Accept {
            actor: actor_id,
            to: Some([follower.ap_id.clone().into()]),
            object,
            kind: Default::default(),
            id,
        };
        let inboxes = vec![follower.shared_inbox_or_inbox()];
        match actor {
            Either::Left(i) => send_ibis_activity(&i, accept, inboxes, context).await?,
            Either::Right(p) => send_ibis_activity(&p, accept, inboxes, context).await?,
        };
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Accept {
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

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        match self.actor.dereference(context).await? {
            Either::Left(instance) => {
                // add to follows
                let person = self.object.actor.dereference_local(context).await?;
                Instance::follow(&person, &instance, false, context)?;
                Ok(())
            }
            Either::Right(_) => Err(anyhow!("person follow not supported").into()),
        }
    }
}
