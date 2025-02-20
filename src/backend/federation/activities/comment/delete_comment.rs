use super::generate_comment_activity_to;
use crate::{
    backend::{
        database::{comment::DbCommentUpdateForm, IbisContext},
        federation::{routes::AnnouncableActivities, send_activity_to_instance},
        utils::{
            error::{BackendError, BackendResult},
            generate_activity_id,
        },
    },
    common::{comment::Comment, instance::Instance, user::Person},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::DeleteType,
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::ActivityHandler,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComment {
    pub(crate) actor: ObjectId<Person>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ObjectId<Comment>,
    #[serde(rename = "type")]
    pub(crate) kind: DeleteType,
    pub(crate) id: Url,
}

impl DeleteComment {
    pub fn new(
        comment: &Comment,
        creator: &Person,
        instance: &Instance,
        context: &Data<IbisContext>,
    ) -> BackendResult<Self> {
        let id = generate_activity_id(context)?;
        Ok(DeleteComment {
            actor: creator.ap_id.clone(),
            object: comment.ap_id.clone(),
            to: generate_comment_activity_to(instance)?,
            kind: Default::default(),
            id,
        })
    }
    pub async fn send(comment: &Comment, context: &Data<IbisContext>) -> BackendResult<()> {
        let instance = Instance::read_for_comment(comment.id, context)?;
        let creator = Person::read(comment.creator_id, context)?;
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
