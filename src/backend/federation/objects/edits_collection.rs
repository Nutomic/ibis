use crate::{
    backend::{
        database::IbisContext,
        federation::objects::edit::ApubEdit,
        utils::error::BackendError,
    },
    common::article::{Article, Edit},
};
use activitypub_federation::{
    config::Data,
    kinds::collection::OrderedCollectionType,
    protocol::verification::verify_domains_match,
    traits::{Collection, Object},
};
use futures::{future, future::try_join_all};
use log::warn;
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
pub struct DbEditCollection();

#[async_trait::async_trait]
impl Collection for DbEditCollection {
    type Owner = Article;
    type DataType = IbisContext;
    type Kind = ApubEditCollection;
    type Error = BackendError;

    async fn read_local(
        article: &Self::Owner,
        context: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let article = Article::read(article.id, context)?;
        let edits = Edit::list_for_article(article.id, context)?;
        let edits = future::try_join_all(
            edits
                .into_iter()
                .map(|e| e.into_json(context))
                .collect::<Vec<_>>(),
        )
        .await?;
        let collection = ApubEditCollection {
            r#type: Default::default(),
            id: Url::from(article.edits_id()?),
            total_items: edits.len() as i32,
            items: edits,
        };
        Ok(collection)
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _context: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(&json.id, expected_domain)?;
        Ok(())
    }

    async fn from_json(
        apub: Self::Kind,
        owner: &Self::Owner,
        context: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        try_join_all(apub.items.into_iter().map(|i| Edit::from_json(i, context)))
            .await
            .map_err(|e| warn!("Failed to synchronize edits for {}: {e}", owner.ap_id))
            .ok();
        Ok(DbEditCollection())
    }
}
