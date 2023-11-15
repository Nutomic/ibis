use crate::error::MyResult;
use crate::federation::objects::instance::DbInstance;
use crate::{database::DatabaseHandle, federation::activities::accept::Accept, generate_object_id};
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
    pub fn new(actor: ObjectId<DbInstance>, object: ObjectId<DbInstance>) -> MyResult<Follow> {
        let id = generate_object_id(actor.inner())?;
        Ok(Follow {
            actor,
            object,
            kind: Default::default(),
            id,
        })
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Follow {
    type DataType = DatabaseHandle;
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
        // add to followers
        let local_instance = {
            let mut instances = data.instances.lock().unwrap();
            let local_instance = instances.first_mut().unwrap();
            local_instance.followers.push(self.actor.inner().clone());
            local_instance.clone()
        };

        // send back an accept
        let follower = self.actor.dereference(data).await?;
        let accept = Accept::new(local_instance.ap_id.clone(), self)?;
        local_instance
            .send(accept, vec![follower.shared_inbox_or_inbox()], data)
            .await?;
        Ok(())
    }
}
