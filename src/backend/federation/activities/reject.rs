use crate::backend::database::conflict::{DbConflict, DbConflictForm};
use crate::backend::database::IbisData;
use crate::backend::error::MyResult;
use crate::backend::federation::objects::edit::ApubEdit;
use crate::backend::utils::generate_activity_id;
use crate::common::DbInstance;
use crate::common::EditVersion;
use activitypub_federation::kinds::activity::RejectType;
use activitypub_federation::{
    config::Data, fetch::object_id::ObjectId, protocol::helpers::deserialize_one_or_many,
    traits::ActivityHandler,
};

use crate::backend::federation::send_activity;
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
        data: &Data<IbisData>,
    ) -> MyResult<()> {
        let local_instance = DbInstance::read_local_instance(data)?;
        let id = generate_activity_id(&local_instance.ap_id)?;
        let reject = RejectEdit {
            actor: local_instance.ap_id.clone(),
            to: vec![user_instance.ap_id.into_inner()],
            object: edit,
            kind: Default::default(),
            id,
        };
        send_activity(
            &local_instance,
            reject,
            vec![Url::parse(&user_instance.inbox_url)?],
            data,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for RejectEdit {
    type DataType = IbisData;
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
        // cant convert this to DbEdit as it tries to apply patch and fails
        let article = self.object.object.dereference(data).await?;
        let creator = self.object.attributed_to.dereference(data).await?;
        let form = DbConflictForm {
            hash: EditVersion::new(&self.object.content),
            diff: self.object.content,
            summary: self.object.summary,
            creator_id: creator.id,
            article_id: article.id,
            previous_version_id: self.object.previous_version,
        };
        DbConflict::create(&form, data)?;
        Ok(())
    }
}
