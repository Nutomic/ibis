use super::{article::ApubArticle, comment::ApubComment};
use crate::{
    backend::{
        database::IbisData,
        utils::error::{Error, MyResult},
    },
    common::{article::DbArticle, comment::DbComment},
};
use activitypub_federation::{config::Data, traits::Object};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use url::Url;

#[derive(Clone, Debug)]
pub enum DbArticleOrComment {
    Article(DbArticle),
    Comment(DbComment),
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum ApubArticleOrComment {
    Article(Box<ApubArticle>),
    Comment(Box<ApubComment>),
}

#[async_trait::async_trait]
impl Object for DbArticleOrComment {
    type DataType = IbisData;
    type Kind = ApubArticleOrComment;
    type Error = Error;

    fn last_refreshed_at(&self) -> Option<DateTime<Utc>> {
        None
    }

    async fn read_from_id(object_id: Url, data: &Data<Self::DataType>) -> MyResult<Option<Self>> {
        let post = DbArticle::read_from_id(object_id.clone(), data).await?;
        Ok(match post {
            Some(o) => Some(Self::Article(o)),
            None => DbComment::read_from_id(object_id, data)
                .await?
                .map(Self::Comment),
        })
    }

    async fn delete(self, data: &Data<Self::DataType>) -> MyResult<()> {
        match self {
            Self::Article(p) => p.delete(data).await,
            Self::Comment(c) => c.delete(data).await,
        }
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> MyResult<Self::Kind> {
        Ok(match self {
            Self::Article(p) => Self::Kind::Article(Box::new(p.into_json(data).await?)),
            Self::Comment(c) => Self::Kind::Comment(Box::new(c.into_json(data).await?)),
        })
    }

    async fn verify(
        apub: &Self::Kind,
        expected_domain: &Url,
        data: &Data<Self::DataType>,
    ) -> MyResult<()> {
        match apub {
            Self::Kind::Article(a) => DbArticle::verify(a, expected_domain, data).await,
            Self::Kind::Comment(a) => DbComment::verify(a, expected_domain, data).await,
        }
    }

    async fn from_json(apub: Self::Kind, context: &Data<Self::DataType>) -> MyResult<Self> {
        Ok(match apub {
            Self::Kind::Article(p) => Self::Article(DbArticle::from_json(*p, context).await?),
            Self::Kind::Comment(n) => Self::Comment(DbComment::from_json(*n, context).await?),
        })
    }
}
