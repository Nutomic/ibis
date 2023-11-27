use crate::database::DatabaseHandle;
use crate::error::MyResult;

use crate::federation::objects::edit::{ApubEdit, DbEdit};
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_activity_id;
use activitypub_federation::kinds::activity::UpdateType;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};
use diffy::{apply, Patch};

use crate::federation::activities::reject::RejectEdit;
use crate::federation::activities::update_local_article::UpdateLocalArticle;
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
        data: &Data<DatabaseHandle>,
    ) -> MyResult<()> {
        let local_instance = data.local_instance();
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let update = UpdateRemoteArticle {
            actor: local_instance.ap_id.clone(),
            to: vec![article_instance.ap_id.into_inner()],
            object: edit.into_json(data).await?,
            kind: Default::default(),
            id,
        };
        dbg!(&update);
        local_instance
            .send(update, vec![article_instance.inbox], data)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UpdateRemoteArticle {
    type DataType = DatabaseHandle;
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
        let edit = DbEdit::from_json(self.object.clone(), data).await?;
        let article_text = {
            let lock = data.articles.lock().unwrap();
            lock.get(self.object.object.inner()).unwrap().text.clone()
        };
        let patch = Patch::from_str(&edit.diff)?;

        match apply(&article_text, &patch) {
            Ok(applied) => {
                let article = {
                    let mut lock = data.articles.lock().unwrap();
                    let article = lock.get_mut(edit.article_id.inner()).unwrap();
                    article.text = applied;
                    article.clone()
                };
                UpdateLocalArticle::send(article, data).await?;
            }
            Err(_e) => {
                let user_instance = self.actor.dereference(data).await?;
                RejectEdit::send(self.object.clone(), user_instance, data).await?;
            }
        }

        Ok(())
    }
}
