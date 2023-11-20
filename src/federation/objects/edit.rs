use crate::database::DatabaseHandle;
use crate::error::Error;
use crate::federation::objects::article::DbArticle;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::traits::Object;
use serde::{Deserialize, Serialize};
use url::Url;

/// Represents a single change to the article.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbEdit {
    pub id: ObjectId<DbEdit>,
    pub diff: String,
    pub local: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EditType {
    Edit,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubEdit {
    #[serde(rename = "type")]
    kind: EditType,
    id: ObjectId<DbEdit>,
    article_id: ObjectId<DbArticle>,
    diff: String,
}

#[async_trait::async_trait]
impl Object for DbEdit {
    type DataType = DatabaseHandle;
    type Kind = ApubEdit;
    type Error = Error;

    async fn read_from_id(
        _object_id: Url,
        _data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        todo!()
    }

    async fn into_json(self, _data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        todo!()
    }

    async fn verify(
        _json: &Self::Kind,
        _expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    async fn from_json(
        _json: Self::Kind,
        _data: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        todo!()
    }
}
