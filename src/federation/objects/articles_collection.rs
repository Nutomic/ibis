use crate::database::DatabaseHandle;
use crate::error::Error;
use crate::federation::objects::article::{ApubArticle, DbArticle};
use crate::federation::objects::instance::DbInstance;

use activitypub_federation::kinds::collection::CollectionType;
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
pub struct ArticleCollection {
    pub(crate) r#type: CollectionType,
    pub(crate) id: Url,
    pub(crate) total_items: i32,
    pub(crate) items: Vec<ApubArticle>,
}

#[derive(Clone, Debug)]
pub struct DbArticleCollection(Vec<DbArticle>);

#[async_trait::async_trait]
impl Collection for DbArticleCollection {
    type Owner = DbInstance;
    type DataType = DatabaseHandle;
    type Kind = ArticleCollection;
    type Error = Error;

    async fn read_local(
        _owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let local_articles = {
            let articles = data.articles.lock().unwrap();
            articles
                .iter()
                .map(|a| a.1)
                .filter(|a| a.local)
                .clone()
                .cloned()
                .collect::<Vec<_>>()
        };
        let articles = future::try_join_all(
            local_articles
                .into_iter()
                .map(|a| a.into_json(data))
                .collect::<Vec<_>>(),
        )
        .await?;
        let collection = ArticleCollection {
            r#type: Default::default(),
            id: data.local_instance().articles_id.into(),
            total_items: articles.len() as i32,
            items: articles,
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
        let articles = try_join_all(
            apub.items
                .into_iter()
                .map(|i| DbArticle::from_json(i, data)),
        )
        .await?;

        // TODO: return value propably not needed
        Ok(DbArticleCollection(articles))
    }
}
