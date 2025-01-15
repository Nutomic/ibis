use crate::{
    backend::{
        database::{article::DbArticleForm, IbisData},
        federation::objects::edits_collection::DbEditCollection,
        utils::error::Error,
    },
    common::{
        article::{DbArticle, EditVersion},
        instance::DbInstance,
    },
};
use activitypub_federation::{
    config::Data,
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    kinds::{object::ArticleType, public},
    protocol::{
        helpers::deserialize_one_or_many,
        verification::{verify_domains_match, verify_is_remote_object},
    },
    traits::Object,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubArticle {
    #[serde(rename = "type")]
    pub kind: ArticleType,
    pub id: ObjectId<DbArticle>,
    pub attributed_to: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    pub edits: CollectionId<DbEditCollection>,
    latest_version: EditVersion,
    content: String,
    name: String,
    protected: bool,
}

#[async_trait::async_trait]
impl Object for DbArticle {
    type DataType = IbisData;
    type Kind = ApubArticle;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let article = DbArticle::read_from_ap_id(&object_id.into(), data).ok();
        Ok(article)
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let local_instance = DbInstance::read_local_instance(data)?;
        Ok(ApubArticle {
            kind: Default::default(),
            id: self.ap_id.clone(),
            attributed_to: local_instance.ap_id.clone(),
            to: vec![public(), local_instance.followers_url()?],
            edits: self.edits_id()?,
            latest_version: self.latest_edit_version(data)?,
            content: self.text,
            name: self.title,
            protected: self.protected,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        verify_is_remote_object(&json.id, data)?;
        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let instance = json.attributed_to.dereference(data).await?;
        let form = DbArticleForm {
            title: json.name,
            text: json.content,
            ap_id: json.id,
            local: false,
            instance_id: instance.id,
            protected: json.protected,
            approved: true,
        };
        let article = DbArticle::create_or_update(form, data)?;

        json.edits.dereference(&article, data).await?;

        Ok(article)
    }
}
