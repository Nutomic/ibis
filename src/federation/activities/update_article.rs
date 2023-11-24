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

use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateArticle {
    pub actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubEdit,
    #[serde(rename = "type")]
    pub kind: CreateType,
    pub id: Url,
}

impl UpdateArticle {
    pub async fn send_to_followers(
        edit: DbEdit,
        article: DbArticle,
        data: &Data<DatabaseHandle>,
    ) -> MyResult<()> {
        debug_assert!(article.local);
        let local_instance = data.local_instance();
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let update = UpdateArticle {
            actor: local_instance.ap_id.clone(),
            to: local_instance.follower_ids(),
            object: edit.into_json(data).await?,
            kind: Default::default(),
            id,
        };
        local_instance.send_to_followers(update, data).await?;
        Ok(())
    }

    pub async fn send_to_origin(
        edit: DbEdit,
        article_instance: DbInstance,
        data: &Data<DatabaseHandle>,
    ) -> MyResult<()> {
        let local_instance = data.local_instance();
        let id = generate_activity_id(local_instance.ap_id.inner())?;
        let update = UpdateArticle {
            actor: local_instance.ap_id.clone(),
            to: vec![article_instance.ap_id.into_inner()],
            object: edit.into_json(data).await?,
            kind: Default::default(),
            id,
        };
        local_instance
            .send(update, vec![article_instance.inbox], data)
            .await?;
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
            DbEdit::from_json(self.object.clone(), data).await?;
            let lock = data.articles.lock().unwrap();
            let article = lock.get(self.object.object.inner()).unwrap();
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
