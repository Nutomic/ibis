use crate::database::instance::DbInstance;
use crate::error::MyResult;
use crate::utils::generate_activity_id;
use crate::{database::MyDataHandle, federation::activities::follow::Follow};
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
    pub fn new(actor: ObjectId<DbInstance>, object: Follow) -> MyResult<Accept> {
        let id = generate_activity_id(actor.inner())?;
        Ok(Accept {
            actor,
            object,
            kind: Default::default(),
            id,
        })
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
        // add to follows
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        let actor = self.actor.dereference(data).await?;
        DbInstance::follow(local_instance.id, actor.id, false, data)?;
        Ok(())
    }
}
