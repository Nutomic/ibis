use crate::{
    backend::{
        database::IbisContext,
        federation::{activities::accept::Accept, send_activity},
        generate_activity_id,
        utils::error::{BackendError, BackendResult},
    },
    common::{instance::DbInstance, user::DbPerson},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FollowType,
    protocol::verification::verify_urls_match,
    traits::{ActivityHandler, Actor},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    pub actor: ObjectId<DbPerson>,
    pub object: ObjectId<DbInstance>,
    #[serde(rename = "type")]
    kind: FollowType,
    id: Url,
}

impl Follow {
    pub async fn send(
        actor: DbPerson,
        to: &DbInstance,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let id = generate_activity_id(context)?;
        let follow = Follow {
            actor: actor.ap_id.clone(),
            object: to.ap_id.clone(),
            kind: Default::default(),
            id,
        };
        send_activity(&actor, follow, vec![to.shared_inbox_or_inbox()], context).await?;
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
        let local_instance = DbInstance::read_local(context)?;
        verify_urls_match(self.object.inner(), local_instance.ap_id.inner())?;
        DbInstance::follow(&actor, &local_instance, false, context)?;

        // send back an accept
        Accept::send(local_instance, self, context).await?;
        Ok(())
    }
}
