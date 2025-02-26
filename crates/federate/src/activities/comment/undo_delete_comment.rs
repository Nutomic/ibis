use super::{delete_comment::DeleteComment, generate_comment_activity_to};
use crate::{
    generate_activity_id,
    objects::{comment::CommentWrapper, instance::InstanceWrapper, user::PersonWrapper},
    routes::AnnouncableActivities,
    send_activity_to_instance,
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UndoType,
    protocol::{
        helpers::deserialize_one_or_many,
        verification::{verify_domains_match, verify_urls_match},
    },
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
pub struct UndoDeleteComment {
    pub(crate) actor: ObjectId<PersonWrapper>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: DeleteComment,
    #[serde(rename = "type")]
    pub(crate) kind: UndoType,
    pub(crate) id: Url,
}

impl UndoDeleteComment {
    pub async fn send(comment: &CommentWrapper, context: &Data<IbisContext>) -> BackendResult<()> {
        let instance: InstanceWrapper = Instance::read_for_comment(comment.id, context)?.into();
        let id = generate_activity_id(context)?;
        let creator: PersonWrapper = Person::read(comment.creator_id, context)?.into();
        let object = DeleteComment::new(comment, &creator, &instance, context)?;
        let activity = UndoDeleteComment {
            actor: creator.ap_id.clone().into(),
            object,
            to: generate_comment_activity_to(&instance)?,
            kind: Default::default(),
            id,
        };
        let activity = AnnouncableActivities::UndoDeleteComment(activity);
        send_activity_to_instance(&creator, activity, &instance, context).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoDeleteComment {
    type DataType = IbisContext;
    type Error = BackendError;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
        verify_domains_match(self.actor.inner(), &self.id)?;
        Ok(())
    }

    async fn receive(self, context: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let form = DbCommentUpdateForm {
            deleted: Some(false),
            updated: Some(Utc::now()),
            ..Default::default()
        };
        let comment = self.object.object.dereference(context).await?;
        Comment::update(form, comment.id, context)?;

        let instance = Instance::read_for_comment(comment.id, context)?;
        if instance.local {
            Self::send(&comment, context).await?;
        }
        Ok(())
    }
}
