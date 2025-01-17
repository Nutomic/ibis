use crate::{
    backend::{
        database::IbisData,
        federation::{objects::comment::ApubComment, send_activity},
        generate_activity_id,
        utils::error::{Error, MyResult},
    },
    common::{
        comment::{CommentView, DbComment},
        instance::DbInstance,
        user::DbPerson,
    },
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::Object,
};
use activitypub_federation::{kinds::public, traits::ActivityHandler};
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
    pub(crate) actor: ObjectId<DbPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub(crate) object: ApubComment,
    #[serde(rename = "type")]
    pub(crate) kind: CreateOrUpdateType,
    pub(crate) id: Url,
}

impl CreateOrUpdateComment {
    pub async fn send(comment: CommentView, data: &Data<IbisData>) -> MyResult<()> {
        let kind = if comment.comment.updated.is_none() {
            CreateOrUpdateType::Create
        } else {
            CreateOrUpdateType::Update
        };
        let recipient = DbInstance::read_for_comment(comment.comment.id, data)?
            .ap_id
            .into_inner();
        let object = comment.comment.into_json(data).await?;
        let id = generate_activity_id(data)?;
        let activity = Self {
            actor: object.attributed_to.clone(),
            object,
            to: vec![public(), recipient.clone()],
            kind,
            id,
        };
        send_activity(&comment.creator, activity, vec![recipient], data).await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for CreateOrUpdateComment {
    type DataType = IbisData;
    type Error = Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        verify_domains_match(&self.id, self.object.id.inner())?;
        verify_domains_match(&self.id, self.actor.inner())?;
        Ok(())
    }

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        DbComment::from_json(self.object, data).await?;

        Ok(())
    }
}
