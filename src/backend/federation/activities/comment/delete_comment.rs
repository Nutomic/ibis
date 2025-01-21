use super::generate_comment_activity_to;
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
    pub(crate) actor: ObjectId<DbPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ObjectId<DbComment>,
    #[serde(rename = "type")]
    pub(crate) kind: DeleteType,
    pub(crate) id: Url,
}

impl DeleteComment {
    pub fn new(
        comment: &DbComment,
        creator: &DbPerson,
        instance: &DbInstance,
        data: &Data<IbisData>,
    ) -> MyResult<Self> {
        let id = generate_activity_id(data)?;
        Ok(DeleteComment {
            actor: creator.ap_id.clone(),
            object: comment.ap_id.clone(),
            to: generate_comment_activity_to(instance)?,
            kind: Default::default(),
            id,
        })
    }
    pub async fn send(comment: &DbComment, data: &Data<IbisData>) -> MyResult<()> {
        let instance = DbInstance::read_for_comment(comment.id, data)?;
        let creator = DbPerson::read(comment.creator_id, data)?;
        let activity = Self::new(comment, &creator, &instance, data)?;
        let activity = AnnouncableActivities::DeleteComment(activity);
        send_activity_to_instance(&creator, activity, &instance, data).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for DeleteComment {
    type DataType = IbisData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(self.actor.inner(), &self.id)?;
        verify_domains_match(self.actor.inner(), self.object.inner())?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let form = DbCommentUpdateForm {
            deleted: Some(true),
            updated: Some(Utc::now()),
            ..Default::default()
        };
        let comment = self.object.dereference(data).await?;
        DbComment::update(form, comment.id, data)?;

        let instance = DbInstance::read_for_comment(comment.id, data)?;
        if instance.local {
            Self::send(&comment, data).await?;
        }
        Ok(())
    }
}
