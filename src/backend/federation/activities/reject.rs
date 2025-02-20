use crate::{
    backend::{
        database::{
            conflict::{DbConflict, DbConflictForm},
            IbisContext,
        },
        federation::{objects::edit::ApubEdit, send_activity},
        utils::{
            error::{BackendError, BackendResult},
            generate_activity_id,
        },
    },
    common::{article::EditVersion, instance::Instance},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::RejectType,
    protocol::helpers::deserialize_one_or_many,
    traits::ActivityHandler,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RejectEdit {
    pub actor: ObjectId<Instance>,
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
        user_instance: Instance,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let local_instance = Instance::read_local(context)?;
        let id = generate_activity_id(context)?;
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
            context,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for RejectEdit {
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
        // Force fetch the article to ensure we have the latest edit that it conflicted with
        let article = self.object.object.dereference_forced(context).await?;
        let creator = self.object.attributed_to.dereference(context).await?;
        let form = DbConflictForm {
            hash: EditVersion::new(&self.object.content),
            diff: self.object.content,
            summary: self.object.summary,
            creator_id: creator.id,
            article_id: article.id,
            previous_version_id: self.object.previous_version,
        };
        DbConflict::create(&form, context)?;
        Ok(())
    }
}
