use crate::backend::database::instance::DbInstance;
use crate::backend::database::MyDataHandle;
use crate::backend::error::Error;
use crate::backend::federation::objects::article::ApubArticle;

use crate::common::DbArticle;
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
    pub r#type: CollectionType,
    pub id: Url,
    pub total_items: i32,
    pub items: Vec<ApubArticle>,
}

#[derive(Clone, Debug)]
pub struct DbArticleCollection(Vec<DbArticle>);

#[async_trait::async_trait]
impl Collection for DbArticleCollection {
    type Owner = DbInstance;
    type DataType = MyDataHandle;
    type Kind = ArticleCollection;
    type Error = Error;

    async fn read_local(
        owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let local_articles = DbArticle::read_all_local(&data.db_connection)?;
        let articles = future::try_join_all(
            local_articles
                .into_iter()
                .map(|a| a.into_json(data))
                .collect::<Vec<_>>(),
        )
        .await?;
        let collection = ArticleCollection {
            r#type: Default::default(),
            id: owner.articles_url.clone().into(),
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
