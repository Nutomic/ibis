use crate::database::DatabaseHandle;
use crate::error::MyResult;
use crate::federation::objects::article::DbArticle;
use crate::federation::objects::edit::{ApubEdit, DbEdit};
use crate::federation::objects::instance::DbInstance;
use crate::utils::generate_activity_id;
use activitypub_federation::kinds::activity::CreateType;
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
pub struct UpdateArticle {
    pub actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ObjectId<DbArticle>,
    pub result: ApubEdit,
    #[serde(rename = "type")]
    pub kind: CreateType,
    pub id: Url,
}

impl UpdateArticle {
    pub async fn send_to_followers(
        article: DbArticle,
        edit: DbEdit,
        data: &Data<DatabaseHandle>,
    ) -> MyResult<()> {
        let local_instance = data.local_instance();
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        if article.local {
            let update = UpdateArticle {
                actor: local_instance.ap_id.clone(),
                to: local_instance.follower_ids(),
                object: article.ap_id,
                result: edit.into_json(data).await?,
                kind: Default::default(),
                id,
            };
            local_instance.send_to_followers(update, data).await?;
        } else {
            let article_instance = article.instance.dereference(data).await?;
            let update = UpdateArticle {
                actor: local_instance.ap_id.clone(),
                to: vec![article_instance.ap_id.into_inner()],
                object: article.ap_id,
                result: edit.into_json(data).await?,
                kind: Default::default(),
                id,
            };
            local_instance
                .send(update, vec![article_instance.inbox], data)
                .await?;
        }
        Ok(())
    }
}
#[async_trait::async_trait]
impl ActivityHandler for UpdateArticle {
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

    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article_local = {
            let edit = DbEdit::from_json(self.result.clone(), data).await?;
            let mut lock = data.articles.lock().unwrap();
            let article = lock.get_mut(self.object.inner()).unwrap();
            article.edits.push(edit);
            // TODO: probably better to apply patch inside DbEdit::from_json()
            let patch = Patch::from_str(&self.result.diff)?;
            article.text = apply(&article.text, &patch)?;
            article.local
        };

        if article_local {
            // No need to wrap in announce, we can construct a new activity as all important info
            // is in the object and result fields.
            let local_instance = data.local_instance();
            let id = generate_activity_id(local_instance.ap_id.inner())?;
            let update = UpdateArticle {
                actor: local_instance.ap_id.clone(),
                to: local_instance.follower_ids(),
                object: self.object,
                result: self.result,
                kind: Default::default(),
                id,
            };
            data.local_instance()
                .send_to_followers(update, data)
                .await?;
        }

        Ok(())
    }
}
