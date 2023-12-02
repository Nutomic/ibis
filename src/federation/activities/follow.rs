use crate::database::instance::DbInstance;
use crate::error::MyResult;
use crate::{database::MyDataHandle, federation::activities::accept::Accept, generate_activity_id};
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
    pub(crate) actor: ObjectId<DbInstance>,
    pub(crate) object: ObjectId<DbInstance>,
    #[serde(rename = "type")]
    kind: FollowType,
    id: Url,
}

impl Follow {
    pub async fn send(
        local_instance: DbInstance,
        to: DbInstance,
        data: &Data<MyDataHandle>,
    ) -> MyResult<()> {
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let follow = Follow {
            actor: local_instance.ap_id.clone(),
            object: to.ap_id.clone(),
            kind: Default::default(),
            id,
        };
        local_instance
            .send(follow, vec![to.shared_inbox_or_inbox()], data)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
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
        let actor = self.actor.dereference(data).await?;
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        DbInstance::follow(actor.id, local_instance.id, false, &data)?;

        // send back an accept
        let follower = self.actor.dereference(data).await?;
        let accept = Accept::new(local_instance.ap_id.clone(), self)?;
        local_instance
            .send(accept, vec![follower.shared_inbox_or_inbox()], data)
            .await?;
        Ok(())
    }
}
