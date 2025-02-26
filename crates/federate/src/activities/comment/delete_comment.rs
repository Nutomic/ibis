use super::generate_comment_activity_to;
use crate::{
    generate_activity_id,
    objects::{comment::CommentWrapper, instance::InstanceWrapper, user::PersonWrapper},
    routes::AnnouncableActivities,
    send_activity_to_instance,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::DeleteType,
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::ActivityHandler,
};
use chrono::Utc;
use ibis_database::{
    common::{comment::Comment, instance::Instance, user::Person},
    error::{BackendError, BackendResult},
    impls::{IbisContext, comment::DbCommentUpdateForm},
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComment {
    pub(crate) actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ObjectId<CommentWrapper>,
    #[serde(rename = "type")]
    pub(crate) kind: DeleteType,
    pub(crate) id: Url,
}

impl DeleteComment {
    pub fn new(
        comment: &CommentWrapper,
        creator: &PersonWrapper,
        instance: &InstanceWrapper,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let id = generate_activity_id(context)?;
        Ok(DeleteComment {
            actor: creator.ap_id.clone().into(),
            object: comment.ap_id.clone().into(),
            to: generate_comment_activity_to(instance)?,
            kind: Default::default(),
            id,
        })
    }
    pub async fn send(comment: &CommentWrapper, context: &Data<IbisContext>) -> BackendResult<()> {
        let instance: InstanceWrapper = Instance::read_for_comment(comment.id, context)?.into();
        let creator: PersonWrapper = Person::read(comment.creator_id, context)?.into();
        let activity = Self::new(comment, &creator, &instance, context)?;
        let activity = AnnouncableActivities::DeleteComment(activity);
        send_activity_to_instance(&creator, activity, &instance, context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for DeleteComment {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(self.actor.inner(), &self.id)?;
        verify_domains_match(self.actor.inner(), self.object.inner())?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let form = DbCommentUpdateForm {
            deleted: Some(true),
            updated: Some(Utc::now()),
            ..Default::default()
        };
        let comment = self.object.dereference(context).await?;
        Comment::update(form, comment.id, context)?;

        let instance = Instance::read_for_comment(comment.id, context)?;
        if instance.local {
            Self::send(&comment, context).await?;
        }
        Ok(())
    }
}
