use crate::federation::objects::instance::DbInstance;
use crate::{database::DatabaseHandle, error::Error};
use activitypub_federation::kinds::object::ArticleType;
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::public,
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::Object,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbArticle {
    pub title: String,
    pub text: String,
    pub ap_id: ObjectId<DbArticle>,
    pub instance: ObjectId<DbInstance>,
    pub local: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    #[serde(rename = "type")]
    kind: ArticleType,
    id: ObjectId<DbArticle>,
    pub(crate) attributed_to: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    content: String,
    name: String,
}

#[async_trait::async_trait]
impl Object for DbArticle {
    type DataType = DatabaseHandle;
    type Kind = Article;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let posts = data.articles.lock().unwrap();
        let res = posts
            .clone()
            .into_iter()
            .find(|u| u.1.ap_id.inner() == &object_id)
            .map(|u| u.1);
        Ok(res)
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let instance = self.instance.dereference_local(data).await?;
        Ok(Article {
            kind: Default::default(),
            id: self.ap_id,
            attributed_to: self.instance,
            to: vec![public(), instance.followers_url()?],
            content: self.text,
            name: self.title,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let article = DbArticle {
            title: json.name,
            text: json.content,
            ap_id: json.id,
            instance: json.attributed_to,
            local: false,
        };

        let mut lock = data.articles.lock().unwrap();
        lock.insert(article.ap_id.inner().clone(), article.clone());
        Ok(article)
    }
}
