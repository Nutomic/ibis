use crate::{
    backend::{
        database::IbisData,
        error::{Error, MyResult},
        federation::objects::article::ApubArticle,
    },
    common::{utils::http_protocol_str, DbArticle},
};
use activitypub_federation::{
    config::Data,
    fetch::collection_id::CollectionId,
    kinds::collection::CollectionType,
    protocol::verification::verify_domains_match,
    traits::{Collection, Object},
};
use futures::future::{join_all, try_join_all};
use log::warn;
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
pub struct DbArticleCollection(());

pub fn local_articles_url(domain: &str) -> MyResult<CollectionId<DbArticleCollection>> {
    Ok(CollectionId::parse(&format!(
        "{}://{domain}/all_articles",
        http_protocol_str()
    ))?)
}

#[async_trait::async_trait]
impl Collection for DbArticleCollection {
    type Owner = ();
    type DataType = IbisData;
    type Kind = ArticleCollection;
    type Error = Error;

    async fn read_local(
        _owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self::Kind, Self::Error> {
        let local_articles = DbArticle::read_all(Some(true), None, data)?;
        let articles = try_join_all(
            local_articles
                .into_iter()
                .map(|a| a.into_json(data))
                .collect::<Vec<_>>(),
        )
        .await?;
        let collection = ArticleCollection {
            r#type: Default::default(),
            id: local_articles_url(&data.config.federation.domain)?.into(),
            total_items: articles.len() as i32,
            items: articles,
        };
        Ok(collection)
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(&json.id, expected_domain)?;
        Ok(())
    }

    async fn from_json(
        apub: Self::Kind,
        _owner: &Self::Owner,
        data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let articles =
            apub.items
                .into_iter()
                .filter(|i| !i.id.is_local(data))
                .map(|article| async {
                    let id = article.id.clone();
                    let res = DbArticle::from_json(article, data).await;
                    if let Err(e) = &res {
                        warn!("Failed to synchronize article {id}: {e}");
                    }
                    res
                });
        join_all(articles).await;

        Ok(DbArticleCollection(()))
    }
}
