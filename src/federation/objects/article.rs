use crate::database::article::DbArticleForm;
use crate::database::edit::EditVersion;
use crate::database::instance::DbInstance;
use crate::database::{article::DbArticle, MyDataHandle};
use crate::error::Error;
use crate::federation::objects::edits_collection::DbEditCollection;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::kinds::object::ArticleType;
use activitypub_federation::kinds::public;
use activitypub_federation::protocol::verification::verify_domains_match;
use activitypub_federation::{
    fetch::object_id::ObjectId, protocol::helpers::deserialize_one_or_many, traits::Object,
};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubArticle {
    #[serde(rename = "type")]
    pub(crate) kind: ArticleType,
    pub(crate) id: ObjectId<DbArticle>,
    pub(crate) attributed_to: ObjectId<DbInstance>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub(crate) to: Vec<Url>,
    pub edits: CollectionId<DbEditCollection>,
    latest_version: EditVersion,
    content: String,
    name: String,
}

#[async_trait::async_trait]
impl Object for DbArticle {
    type DataType = MyDataHandle;
    type Kind = ApubArticle;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        let article = DbArticle::read_from_ap_id(&object_id.into(), &data.db_connection).ok();
        Ok(article)
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let local_instance = DbInstance::read_local_instance(&data.db_connection)?;
        Ok(ApubArticle {
            kind: Default::default(),
            id: self.ap_id.clone(),
            attributed_to: local_instance.ap_id.clone(),
            to: vec![public(), local_instance.followers_url()?],
            edits: self.edits_id()?,
            latest_version: self.latest_edit_version(&data.db_connection)?,
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
        let instance = json.attributed_to.dereference(data).await?;
        let form = DbArticleForm {
            title: json.name,
            text: json.content,
            ap_id: json.id,
            local: false,
            instance_id: instance.id,
        };
        let article = DbArticle::create(&form, &data.db_connection)?;

        json.edits.dereference(&article, data).await?;

        Ok(article)
    }
}
