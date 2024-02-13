use crate::backend::database::IbisData;
use crate::backend::error::MyResult;
use crate::backend::federation::objects::article::ApubArticle;

use crate::backend::utils::generate_activity_id;
use crate::common::DbInstance;
use activitypub_federation::kinds::activity::UpdateType;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::helpers::deserialize_one_or_many,
    traits::{ActivityHandler, Object},
};

use crate::common::validation::can_edit_article;
use crate::common::DbArticle;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLocalArticle {
    pub actor: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub object: ApubArticle,
    #[serde(rename = "type")]
    pub kind: UpdateType,
    pub id: Url,
}

impl UpdateLocalArticle {
    /// Sent from article origin instance
    pub async fn send(
        article: DbArticle,
        extra_recipients: Vec<DbInstance>,
        data: &Data<IbisData>,
    ) -> MyResult<()> {
        debug_assert!(article.local);
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        let id = generate_activity_id(&local_instance.ap_id)?;
        let mut to = local_instance.follower_ids(data)?;
        to.extend(extra_recipients.iter().map(|i| i.ap_id.inner().clone()));
        let update = UpdateLocalArticle {
            actor: local_instance.ap_id.clone(),
            to,
            object: article.into_json(data).await?,
            kind: Default::default(),
            id,
        };
        local_instance
            .send_to_followers(update, extra_recipients, data)
            .await?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl ActivityHandler for UpdateLocalArticle {
    type DataType = IbisData;
    type Error = crate::backend::error::Error;

    fn id(&self) -> &Url {
        &self.id
    }

    fn actor(&self) -> &Url {
        self.actor.inner()
    }

    async fn verify(&self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        let article = DbArticle::read_from_ap_id(&self.object.id, &data.db_connection)?;
        can_edit_article(&article, false)?;
        Ok(())
    }

    /// Received on article follower instances (where article is always remote)
    async fn receive(self, data: &Data<Self::DataType>) -> Result<(), Self::Error> {
        DbArticle::from_json(self.object, data).await?;

        Ok(())
    }
}
