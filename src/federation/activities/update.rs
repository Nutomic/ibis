use crate::federation::objects::article::DbArticle;
use crate::{database::DatabaseHandle, federation::objects::person::DbUser};
use activitypub_federation::kinds::activity::UpdateType;
use activitypub_federation::{config::Data, fetch::object_id::ObjectId, traits::ActivityHandler};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::MyResult;
use crate::utils::generate_object_id;

/// represents a diff between two strings
#[derive(Deserialize, Serialize, Debug)]
pub struct Diff {}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    actor: ObjectId<DbUser>,
    object: ObjectId<DbArticle>,
    result: Diff,
    #[serde(rename = "type")]
    kind: UpdateType,
    id: Url,
}

impl Update {
    pub fn new(actor: ObjectId<DbUser>, object: ObjectId<DbArticle>) -> MyResult<Update> {
        let id = generate_object_id(actor.inner())?;
        Ok(Update {
            actor,
            object,
            result: Diff {},
            kind: Default::default(),
            id,
        })
    }
}

#[async_trait::async_trait]
impl ActivityHandler for Update {
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

    async fn receive(self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }
}
