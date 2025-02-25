use super::generate_comment_activity_to;
use crate::{
    generate_activity_id,
    objects::{
        comment::{ApubComment, CommentWrapper},
        instance::InstanceWrapper,
        user::PersonWrapper,
    },
    routes::AnnouncableActivities,
    send_activity_to_instance,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::{ActivityHandler, Object},
};
use ibis_database::{
    common::{instance::Instance, user::Person},
    error::{BackendError, BackendResult},
    impls::IbisContext,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum CreateOrUpdateType {
    Create,
    Update,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrUpdateComment {
    pub(crate) actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ApubComment,
    #[serde(rename = "type")]
    pub(crate) kind: CreateOrUpdateType,
    pub(crate) id: Url,
}

impl CreateOrUpdateComment {
    pub async fn send(comment: &CommentWrapper, context: &Data<IbisContext>) -> BackendResult<()> {
        let instance: InstanceWrapper = Instance::read_for_comment(comment.id, context)?.into();

        let kind = if comment.updated.is_none() {
            CreateOrUpdateType::Create
        } else {
            CreateOrUpdateType::Update
        };
        let object = comment.clone().into_json(context).await?;
        let id = generate_activity_id(context)?;
        let activity = Self {
            actor: object.attributed_to.clone(),
            object,
            to: generate_comment_activity_to(&instance)?,
            kind,
            id,
        };
        let activity = AnnouncableActivities::CreateOrUpdateComment(activity);
        let creator: PersonWrapper = Person::read(comment.creator_id, context)?.into();
        send_activity_to_instance(&creator, activity, &instance, context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdateComment {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, self.object.id.inner())?;
        verify_domains_match(&self.id, self.actor.inner())?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let comment = CommentWrapper::from_json(self.object, context).await?;

        let instance = Instance::read_for_comment(comment.id, context)?;
        if instance.local {
            Self::send(&comment, context).await?;
        }
        Ok(())
    }
}
