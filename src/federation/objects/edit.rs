use crate::database::DatabaseHandle;
use crate::error::{Error, MyResult};

use crate::federation::objects::article::DbArticle;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use activitypub_federation::traits::Object;
use diffy::create_patch;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use sha2::Sha224;
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EditVersion(String);

impl Default for EditVersion {
    fn default() -> Self {
        let sha224 = Sha224::new();
        let hash = format!("{:X}", sha224.finalize());
        EditVersion(hash)
    }
}

/// Represents a single change to the article.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DbEdit {
    pub id: ObjectId<DbEdit>,
    pub diff: String,
    pub article_id: ObjectId<DbArticle>,
    pub version: EditVersion,
    pub local: bool,
}

impl DbEdit {
    pub fn new(original_article: &DbArticle, updated_text: &str) -> MyResult<Self> {
        let diff = create_patch(&original_article.text, updated_text);
        let mut sha224 = Sha224::new();
        sha224.update(diff.to_bytes());
        let hash = format!("{:X}", sha224.finalize());
        let edit_id = ObjectId::parse(&format!("{}/{}", original_article.ap_id, hash))?;
        Ok(DbEdit {
            id: edit_id,
            diff: diff.to_string(),
            article_id: original_article.ap_id.clone(),
            version: EditVersion(hash),
            local: true,
        })
    }
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
    pub content: String,
    pub version: EditVersion,
    pub object: ObjectId<DbArticle>,
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
        Ok(ApubEdit {
            kind: EditType::Edit,
            id: self.id,
            content: self.diff,
            version: self.version,
            object: self.article_id,
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
        let edit = Self {
            id: json.id,
            diff: json.content,
            article_id: json.object,
            version: json.version,
            local: false,
        };
        let mut lock = data.articles.lock().unwrap();
        let article = lock.get_mut(edit.article_id.inner()).unwrap();
        article.edits.push(edit.clone());
        Ok(edit)
    }
}
