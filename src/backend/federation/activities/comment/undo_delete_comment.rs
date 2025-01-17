use super::{delete_comment::DeleteComment, generate_comment_activity_to};
use crate::{
    backend::{
        database::{comment::DbCommentUpdateForm, IbisData},
        federation::{routes::AnnouncableActivities, send_activity_to_instance},
        utils::{
            error::{Error, MyResult},
            generate_activity_id,
        },
    },
    common::{comment::DbComment, instance::DbInstance, user::DbPerson},
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
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UndoDeleteComment {
    pub(crate) actor: ObjectId<DbPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: DeleteComment,
    #[serde(rename = "type")]
    pub(crate) kind: UndoType,
    pub(crate) id: Url,
}

impl UndoDeleteComment {
    pub async fn send(comment: &DbComment, data: &Data<IbisData>) -> MyResult<()> {
        let instance = DbInstance::read_for_comment(comment.id, data)?;
        let id = generate_activity_id(data)?;
        let creator = DbPerson::read(comment.creator_id, data)?;
        let object = DeleteComment::new(comment, &creator, &instance, data)?;
        let activity = UndoDeleteComment {
            actor: creator.ap_id.clone(),
            object,
            to: generate_comment_activity_to(&instance)?,
            kind: Default::default(),
            id,
        };
        let activity = AnnouncableActivities::UndoDeleteComment(activity);
        send_activity_to_instance(&creator, activity, &instance, data).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UndoDeleteComment {
    type DataType = IbisData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_urls_match(self.actor.inner(), self.object.actor.inner())?;
        verify_domains_match(self.actor.inner(), &self.id)?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let form = DbCommentUpdateForm {
            deleted: Some(false),
            updated: Some(Utc::now()),
            ..Default::default()
        };
        let comment = self.object.object.dereference(data).await?;
        DbComment::update(form, comment.id, data)?;

        let instance = DbInstance::read_for_comment(comment.id, data)?;
        if instance.local {
            Self::send(&comment, data).await?;
        }
        Ok(())
    }
}
