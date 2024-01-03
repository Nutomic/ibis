use crate::backend::database::edit::DbEditForm;
use crate::backend::database::user::DbPerson;
use crate::backend::database::MyDataHandle;
use crate::backend::error::Error;
use crate::common::EditVersion;
use crate::common::{DbArticle, DbEdit};
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::traits::Object;
use serde::{Deserialize, Serialize};
use url::Url;

/// Same type used by Forgefed
/// https://codeberg.org/ForgeFed/ForgeFed/issues/88
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PatchType {
    Patch,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubEdit {
    #[serde(rename = "type")]
    kind: PatchType,
    pub id: ObjectId<DbEdit>,
    pub content: String,
    pub version: EditVersion,
    pub previous_version: EditVersion,
    pub object: ObjectId<DbArticle>,
    pub attributed_to: ObjectId<DbPerson>,
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
        let creator = DbPerson::read(self.creator_id, data)?;
        Ok(ApubEdit {
            kind: PatchType::Patch,
            id: ObjectId::parse(&self.ap_id)?,
            content: self.diff,
            version: self.hash,
            previous_version: self.previous_version_id,
            object: ObjectId::parse(&article.ap_id)?,
            attributed_to: creator.ap_id,
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
        let creator = json.attributed_to.dereference(data).await?;
        let form = DbEditForm {
            creator_id: creator.id,
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
