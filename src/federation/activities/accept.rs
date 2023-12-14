use crate::database::instance::DbInstance;
use crate::error::MyResult;
use crate::federation::send_activity;
use crate::utils::generate_activity_id;
use crate::{database::MyDataHandle, federation::activities::follow::Follow};
use activitypub_federation::traits::Actor;
use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, kinds::activity::AcceptType, traits::ActivityHandler,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
    actor: ObjectId<DbInstance>,
    object: Follow,
    #[serde(rename = "type")]
    kind: AcceptType,
    id: Url,
}

impl Accept {
    pub async fn send(
        local_instance: DbInstance,
        object: Follow,
        data: &Data<MyDataHandle>,
    ) -> MyResult<()> {
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let follower = object.actor.dereference(data).await?;
        let accept = Accept {
            actor: local_instance.ap_id.clone(),
            object,
            kind: Default::default(),
            id,
        };
        dbg!(&accept);
        send_activity(
            &local_instance,
            accept,
            vec![follower.shared_inbox_or_inbox()],
            data,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Accept {
    type DataType = MyDataHandle;
    type Error = crate::error::Error;

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
        dbg!(&self);
        // add to follows
        let person = self.object.actor.dereference_local(data).await?;
        let instance = self.actor.dereference(data).await?;
        DbInstance::follow(&person, &instance, false, data)?;
        Ok(())
    }
}
