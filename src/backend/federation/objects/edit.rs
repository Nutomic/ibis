use crate::{
    backend::{
        database::{edit::DbEditForm, IbisContext},
        utils::error::BackendError,
    },
    common::{
        article::{Article, Edit, EditVersion},
        user::Person,
    },
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    protocol::verification::{verify_domains_match, verify_is_remote_object},
    traits::Object,
};
use chrono::{DateTime, Utc};
use log::warn;
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
    pub id: ObjectId<Edit>,
    pub content: String,
    pub summary: String,
    pub version: EditVersion,
    pub previous_version: EditVersion,
    pub object: ObjectId<Article>,
    pub attributed_to: ObjectId<Person>,
    pub published: DateTime<Utc>,
}

#[async_trait::async_trait]
impl Object for Edit {
    type DataType = IbisContext;
    type Kind = ApubEdit;
    type Error = BackendError;

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(Edit::read_from_ap_id(&object_id.into(), context).ok())
    }

    async fn into_json(self, context: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let article = Article::read_view(self.article_id, None, context)?;
        let creator = Person::read(self.creator_id, context)?;
        Ok(ApubEdit {
            kind: PatchType::Patch,
            id: self.ap_id,
            content: self.diff,
            summary: self.summary,
            version: self.hash,
            previous_version: self.previous_version_id,
            object: article.article.ap_id,
            attributed_to: creator.ap_id,
            published: self.published,
        })
    }

    async fn verify(
        json: &Self::Kind,
        expected_domain: &Url,
        context: &Data<Self::DataType>,
    ) -> Result<(), Self::Error> {
        verify_domains_match(json.id.inner(), expected_domain)?;
        verify_is_remote_object(&json.id, context)?;
        Ok(())
    }

    async fn from_json(
        json: Self::Kind,
        context: &Data<Self::DataType>,
    ) -> Result<Self, Self::Error> {
        let article = json.object.dereference(context).await?;
        let creator = match json.attributed_to.dereference(context).await {
            Ok(c) => c,
            Err(e) => {
                // If actor couldnt be fetched, use ghost as placeholder
                warn!("Failed to fetch user {}: {e}", json.attributed_to);
                Person::ghost(context)?
            }
        };
        let form = DbEditForm {
            creator_id: creator.id,
            ap_id: json.id,
            diff: json.content,
            summary: json.summary,
            article_id: article.id,
            hash: json.version,
            previous_version_id: json.previous_version,
            published: json.published,
            pending: false,
        };
        let edit = Edit::create(&form, context)?;
        Ok(edit)
    }
}
