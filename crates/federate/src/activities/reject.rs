use crate::{
    generate_activity_id,
    objects::{edit::ApubEdit, instance::InstanceWrapper},
    send_activity,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::RejectType,
    protocol::helpers::deserialize_one_or_many,
    traits::ActivityHandler,
};
use ibis_database::{
    common::{
        article::{Conflict, EditVersion},
        instance::Instance,
    },
    error::{BackendError, BackendResult},
    impls::{IbisContext, conflict::DbConflictForm},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RejectEdit {
    pub actor: ObjectId<InstanceWrapper>,
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
        user_instance: InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<()> {
        let local_instance: InstanceWrapper = Instance::read_local(context)?.into();
        let id = generate_activity_id(context)?;
        let reject = RejectEdit {
            actor: local_instance.ap_id.clone().into(),
            to: vec![user_instance.ap_id.clone().into()],
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
        Conflict::create(&form, context)?;
        Ok(())
    }
}
