use crate::database::article::DbArticle;
use crate::database::edit::{DbEdit, DbEditForm};
use crate::database::version::EditVersion;
use crate::database::MyDataHandle;
use crate::error::Error;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::traits::Object;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EditType {
    Edit,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubEdit {
    #[serde(rename = "type")]
    kind: EditType,
    pub id: ObjectId<DbEdit>,
    pub content: String,
    pub version: EditVersion,
    pub previous_version: EditVersion,
    pub object: ObjectId<DbArticle>,
}

#[async_trait::async_trait]
impl Object for DbEdit {
    type DataType = MyDataHandle;
    type Kind = ApubEdit;
    type Error = Error;

    async fn read_from_id(
        object_id: Url,
        data: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(DbEdit::read_from_ap_id(&object_id.into(), data).ok())
    }

    async fn into_json(self, data: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let article = DbArticle::read(self.article_id, &data.db_connection)?;
        Ok(ApubEdit {
            kind: EditType::Edit,
            id: self.ap_id,
            content: self.diff,
            version: self.hash,
            previous_version: self.previous_version_id,
            object: article.ap_id,
        })
    }

    async fn verify(
        _json: &Self::Kind,
        _expected_domain: &Url,
        _data: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn from_json(json: Self::Kind, data: &Data<Self::DataType>) -> Result<Self, Self::Error> {
        let article = json.object.dereference(data).await?;
        let form = DbEditForm {
            ap_id: json.id,
            diff: json.content,
            article_id: article.id,
            hash: json.version,
            previous_version_id: json.previous_version,
        };
        let edit = DbEdit::create(&form, &data.db_connection)?;
        Ok(edit)
    }
}
