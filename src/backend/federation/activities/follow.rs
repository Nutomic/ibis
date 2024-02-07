use crate::backend::error::MyResult;
use crate::backend::federation::send_activity;
use crate::backend::{
    database::MyDataHandle, federation::activities::accept::Accept, generate_activity_id,
};
use crate::common::DbInstance;
use crate::common::DbPerson;
use activitypub_federation::protocol::verification::verify_urls_match;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::FollowType,
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
    pub async fn send(actor: DbPerson, to: DbInstance, data: &Data<MyDataHandle>) -> MyResult<()> {
        let id = generate_activity_id(actor.ap_id.inner())?;
        let follow = Follow {
            actor: actor.ap_id.clone(),
            object: to.ap_id.clone(),
            kind: Default::default(),
            id,
        };
        send_activity(&actor, follow, vec![to.shared_inbox_or_inbox()], data).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
    type DataType = MyDataHandle;
    type Error = crate::backend::error::Error;

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
        let actor = self.actor.dereference(data).await?;
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        verify_urls_match(self.object.inner(), local_instance.ap_id.inner())?;
        DbInstance::follow(&actor, &local_instance, false, data)?;

        // send back an accept
        Accept::send(local_instance, self, data).await?;
        Ok(())
    }
}
