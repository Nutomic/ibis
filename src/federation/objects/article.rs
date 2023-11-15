use crate::federation::objects::instance::DbInstance;
use crate::{database::DatabaseHandle, error::Error, generate_object_id};
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

#[derive(Clone, Debug)]
pub struct DbArticle {
    pub text: String,
    pub ap_id: ObjectId<DbArticle>,
    pub instance: ObjectId<DbInstance>,
    pub local: bool,
}

impl DbArticle {
    pub fn new(text: String, attributed_to: ObjectId<DbInstance>) -> Result<DbArticle, Error> {
        let ap_id = generate_object_id(attributed_to.inner().domain().unwrap())?.into();
        Ok(DbArticle {
            text,
            ap_id,
            instance: attributed_to,
            local: true,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Article {
    #[serde(rename = "type")]
    kind: ArticleType,
    id: ObjectId<DbArticle>,
    pub(crate) attributed_to: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    content: String,
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
        let posts = data.posts.lock().unwrap();
        let res = posts
            .clone()
            .into_iter()
            .find(|u| u.ap_id.inner() == &object_id);
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
        let post = DbArticle {
            text: json.content,
            ap_id: json.id,
            instance: json.attributed_to,
            local: false,
        };

        let mut lock = data.posts.lock().unwrap();
        lock.push(post.clone());
        Ok(post)
    }
}
