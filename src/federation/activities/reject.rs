use crate::database::instance::DbInstance;
use crate::database::MyDataHandle;
use crate::error::MyResult;
use crate::federation::objects::edit::ApubEdit;
use crate::utils::generate_activity_id;
use activitypub_federation::kinds::activity::RejectType;
use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, protocol::helpers::deserialize_one_or_many,
    traits::ActivityHandler,
};
use rand::random;

use crate::database::DbConflict;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RejectEdit {
    pub actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubEdit,
    #[serde(rename = "type")]
    pub kind: RejectType,
    pub id: Url,
}

impl RejectEdit {
    pub async fn send(
        edit: ApubEdit,
        user_instance: DbInstance,
        data: &Data<MyDataHandle>,
    ) -> MyResult<()> {
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let reject = RejectEdit {
            actor: local_instance.ap_id.clone(),
            to: vec![user_instance.ap_id.into_inner()],
            object: edit,
            kind: Default::default(),
            id,
        };
        local_instance
            .send(reject, vec![Url::parse(&user_instance.inbox_url)?], data)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for RejectEdit {
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
        // cant convert this to DbEdit as it tries to apply patch and fails
        let mut lock = data.conflicts.lock().unwrap();
        let conflict = DbConflict {
            id: random(),
            diff: self.object.content,
            article_id: self.object.object,
            previous_version: self.object.previous_version,
        };
        lock.push(conflict);
        Ok(())
    }
}
