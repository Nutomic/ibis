use crate::database::DatabaseHandle;
use crate::error::MyResult;
use crate::federation::objects::edit::ApubEdit;
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_activity_id;
use activitypub_federation::kinds::activity::RejectType;
use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, protocol::helpers::deserialize_one_or_many,
    traits::ActivityHandler,
};
use rand::random;

use crate::api::DbConflict;
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
        data: &Data<DatabaseHandle>,
    ) -> MyResult<()> {
        let local_instance = data.local_instance();
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let reject = RejectEdit {
            actor: local_instance.ap_id.clone(),
            to: vec![user_instance.ap_id.into_inner()],
            object: edit,
            kind: Default::default(),
            id,
        };
        local_instance
            .send(reject, vec![user_instance.inbox], data)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for RejectEdit {
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
