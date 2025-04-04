use super::InstanceOrPerson;
use crate::{
    activities::following::accept::Accept,
    generate_activity_id,
    objects::{instance::InstanceWrapper, user::PersonWrapper},
    send_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FollowType,
    protocol::{helpers::deserialize_skip_error, verification::verify_urls_match},
    traits::{ActivityHandler, Actor},
};
use anyhow::anyhow;
use either::Either;
use ibis_database::{
    common::{instance::Instance, user::Person},
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub actor: ObjectId<PersonWrapper>,
    /// Optional, for compatibility with platforms that always expect recipient field
    #[serde(deserialize_with = "deserialize_skip_error", default)]
    pub(crate) to: Option<[ObjectId<InstanceOrPerson>; 1]>,
    pub object: ObjectId<InstanceOrPerson>,
    #[serde(rename = "type")]
    kind: FollowType,
    id: Url,
}

impl Follow {
    pub fn new(
        actor: &PersonWrapper,
        to: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let id = generate_activity_id(context)?;
        let to: ObjectId<InstanceOrPerson> = to.ap_id.clone().into();
        Ok(Follow {
            actor: actor.ap_id.clone().into(),
            to: Some([to.clone()]),
            object: to,
            kind: Default::default(),
            id,
        })
    }

    pub async fn send(
        actor: &PersonWrapper,
        to: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let follow = Self::new(actor, to, context)?;
        send_activity(actor, follow, vec![to.shared_inbox_or_inbox()], context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
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
        let actor = self.actor.dereference(context).await?;
        let object = self.object.dereference_local(context).await?;
        match object {
            Either::Left(instance) => {
                if !instance.local {
                    return Err(anyhow!("invalid follow").into());
                }
                verify_urls_match(self.object.inner(), instance.ap_id.inner())?;
                Instance::follow(&actor, &instance, false, context)?;
                Accept::send(Either::Left(instance), self, context).await?;
            }
            Either::Right(person) => {
                if !person.local {
                    return Err(anyhow!("invalid follow").into());
                }
                verify_urls_match(self.object.inner(), person.ap_id.inner())?;
                Person::follow(&actor, &person, context)?;
                Accept::send(Either::Right(person), self, context).await?;
            }
        }

        Ok(())
    }
}
