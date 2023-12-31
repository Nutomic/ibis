use crate::database::article::DbArticle;
use crate::database::MyDataHandle;
use crate::error::Error;
use crate::federation::objects::edit::ApubEdit;

use crate::database::edit::DbEdit;
use crate::database::instance::DbInstance;
use activitypub_federation::kinds::collection::OrderedCollectionType;
use activitypub_federation::{
    config::Data,
    traits::{Collection, Object},
};
use futures::future;
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApubEditCollection {
    pub r#type: OrderedCollectionType,
    pub id: Url,
    pub total_items: i32,
    pub items: Vec<ApubEdit>,
}

#[derive(Clone, Debug)]
pub struct DbEditCollection(pub Vec<DbEdit>);

#[async_trait::async_trait]
impl Collection for DbEditCollection {
    type Owner = DbArticle;
    type DataType = MyDataHandle;
    type Kind = ApubEditCollection;
    type Error = Error;

    async fn read_local(
        owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let article = DbArticle::read_view(owner.id, &data.db_connection)?;

        let edits = future::try_join_all(
            article
                .edits
                .into_iter()
                .map(|a| a.into_json(data))
                .collect::<Vec<_>>(),
        )
        .await?;
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        let collection = ApubEditCollection {
            r#type: Default::default(),
            id: Url::from(local_instance.articles_url),
            total_items: edits.len() as i32,
            items: edits,
        };
        Ok(collection)
    }

    async fn verify(
        _apub: &Self::Kind,
        _expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn from_json(
        apub: Self::Kind,
        _owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let edits =
            try_join_all(apub.items.into_iter().map(|i| DbEdit::from_json(i, data))).await?;
        // TODO: return value propably not needed
        Ok(DbEditCollection(edits))
    }
}
