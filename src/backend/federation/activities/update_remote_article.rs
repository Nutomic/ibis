use crate::{
    backend::{
        database::IbisData,
        error::MyResult,
        federation::{
            activities::{reject::RejectEdit, update_local_article::UpdateLocalArticle},
            objects::edit::ApubEdit,
            send_activity,
        },
        utils::generate_activity_id,
    },
    common::{validation::can_edit_article, DbArticle, DbEdit, DbInstance},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::activity::UpdateType,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use diffy::{apply, Patch};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRemoteArticle {
    pub actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubEdit,
    #[serde(rename = "type")]
    pub kind: UpdateType,
    pub id: Url,
}

impl UpdateRemoteArticle {
    /// Sent by a follower instance
    pub async fn send(
        edit: DbEdit,
        article_instance: DbInstance,
        data: &Data<IbisData>,
    ) -> MyResult<()> {
        let local_instance = DbInstance::read_local_instance(data)?;
        let id = generate_activity_id(&local_instance.ap_id)?;
        let update = UpdateRemoteArticle {
            actor: local_instance.ap_id.clone(),
            to: vec![article_instance.ap_id.into_inner()],
            object: edit.into_json(data).await?,
            kind: Default::default(),
            id,
        };
        send_activity(
            &local_instance,
            update,
            vec![Url::parse(&article_instance.inbox_url)?],
            data,
        )
        .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UpdateRemoteArticle {
    type DataType = IbisData;
    type Error = crate::backend::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = DbArticle::read_from_ap_id(&self.object.object, data)?;
        can_edit_article(&article, false)?;
        Ok(())
    }

    /// Received on article origin instances
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let local_article = DbArticle::read_from_ap_id(&self.object.object, data)?;
        let patch = Patch::from_str(&self.object.content)?;

        match apply(&local_article.text, &patch) {
            Ok(applied) => {
                let edit = DbEdit::from_json(self.object.clone(), data).await?;
                let article = DbArticle::update_text(edit.article_id, &applied, data)?;
                UpdateLocalArticle::send(article, vec![self.actor.dereference(data).await?], data)
                    .await?;
            }
            Err(_e) => {
                let user_instance = self.actor.dereference(data).await?;
                RejectEdit::send(self.object.clone(), user_instance, data).await?;
            }
        }

        Ok(())
    }
}
