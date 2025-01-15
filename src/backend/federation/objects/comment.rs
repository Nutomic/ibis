use crate::{
    backend::{
        database::{article::DbArticleForm, IbisData},
       utils::error::Error,
        federation::objects::edits_collection::DbEditCollection,
    },
    common::{DbArticle, DbComment, DbInstance, DbPerson, EditVersion},
};
use activitypub_federation::{
    config::Data,
    fetch::{collection_id::CollectionId, object_id::ObjectId},
    kinds::{
        object::{ArticleType, NoteType},
        public,
    },
    protocol::{helpers::deserialize_one_or_many, verification::verify_domains_match},
    traits::Object,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

use super::article_or_comment::ArticleOrComment;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubComment {
    #[serde(rename = "type")]
    pub kind: NoteType,
    pub id: ObjectId<DbArticle>,
    pub attributed_to: ObjectId<DbPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    content: String,
    pub in_reply_to: ObjectId<ArticleOrComment>,
    pub(crate) published: Option<DateTime<Utc>>,
    pub(crate) updated: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
impl Object for DbComment {
    type DataType = IbisData;
    type Kind = ApubComment;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(DbComment::read_from_ap_id(&object_id.into(), data).ok())
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let local_instance = DbInstance::read_local_instance(data)?;
        Ok(ApubComment {
            todo!()
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
