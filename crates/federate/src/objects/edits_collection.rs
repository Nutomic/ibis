use super::edit::EditWrapper;
use crate::objects::edit::ApubEdit;
use activitypub_federation::{
    config::Data,
    kinds::collection::OrderedCollectionType,
    protocol::verification::verify_domains_match,
    traits::{Collection, Object},
};
use futures::{future, future::try_join_all};
use ibis_database::{
    common::article::{Article, Edit},
    error::BackendError,
    impls::IbisContext,
};
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
pub struct EditCollection();

#[async_trait::async_trait]
impl Collection for EditCollection {
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
                .map(EditWrapper)
                .map(|e| e.into_json(context))
                .collect::<Vec<_>>(),
        )
        .await?;
        let collection = ApubEditCollection {
            r#type: Default::default(),
            id: article.edits_id()?.into(),
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
        try_join_all(
            apub.items
                .into_iter()
                .map(|i| EditWrapper::from_json(i, context)),
        )
        .await
        .map_err(|e| warn!("Failed to synchronize edits for {}: {e}", owner.ap_id))
        .ok();
        Ok(EditCollection())
    }
}
