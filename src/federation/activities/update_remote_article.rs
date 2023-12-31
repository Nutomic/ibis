use crate::database::MyDataHandle;
use crate::error::MyResult;

use crate::database::article::DbArticle;
use crate::database::edit::DbEdit;
use crate::database::instance::DbInstance;
use crate::federation::activities::reject::RejectEdit;
use crate::federation::activities::update_local_article::UpdateLocalArticle;
use crate::federation::objects::edit::ApubEdit;
use crate::federation::send_activity;
use crate::utils::generate_activity_id;
use activitypub_federation::kinds::activity::UpdateType;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
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
        data: &Data<MyDataHandle>,
    ) -> MyResult<()> {
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        let id = generate_activity_id(local_instance.ap_id.inner())?;
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
    type DataType = MyDataHandle;
    type Error = crate::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, _data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        Ok(())
    }

    /// Received on article origin instances
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let local_article = DbArticle::read_from_ap_id(&self.object.object, &data.db_connection)?;
        let patch = Patch::from_str(&self.object.content)?;

        match apply(&local_article.text, &patch) {
            Ok(applied) => {
                let edit = DbEdit::from_json(self.object.clone(), data).await?;
                let article =
                    DbArticle::update_text(edit.article_id, &applied, &data.db_connection)?;
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
