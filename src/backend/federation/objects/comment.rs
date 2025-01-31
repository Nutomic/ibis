use super::article_or_comment::DbArticleOrComment;
use crate::{
    backend::{
        database::{comment::DbCommentInsertForm, IbisContext},
        utils::{error::BackendError, validate::validate_comment_max_depth},
    },
    common::{article::DbArticle, comment::DbComment, user::DbPerson},
};
use activitypub_federation::{
    config::Data,
    fetch::object_id::ObjectId,
    kinds::{object::NoteType, public},
    protocol::{
        helpers::deserialize_one_or_many,
        verification::{verify_domains_match, verify_is_remote_object},
    },
    traits::Object,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ApubComment {
    #[serde(rename = "type")]
    pub kind: NoteType,
    pub id: ObjectId<DbComment>,
    pub attributed_to: ObjectId<DbPerson>,
    #[serde(deserialize_with = "deserialize_one_or_many")]
    pub to: Vec<Url>,
    content: String,
    pub in_reply_to: ObjectId<DbArticleOrComment>,
    pub published: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
}

#[async_trait::async_trait]
impl Object for DbComment {
    type DataType = IbisContext;
    type Kind = ApubComment;
    type Error = BackendError;

    async fn read_from_id(
        object_id: Url,
        context: &Data<Self::DataType>,
    ) -> Result<Option<Self>, Self::Error> {
        Ok(DbComment::read_from_ap_id(&object_id.into(), context).ok())
    }

    async fn into_json(self, context: &Data<Self::DataType>) -> Result<Self::Kind, Self::Error> {
        let creator = DbPerson::read(self.creator_id, context)?;
        let in_reply_to = if let Some(parent_comment_id) = self.parent_id {
            let comment = DbComment::read(parent_comment_id, context)?;
            comment.ap_id.into_inner().into()
        } else {
            let article = DbArticle::read(self.article_id, context)?;
            article.ap_id.into_inner().into()
        };
        Ok(ApubComment {
            kind: NoteType::Note,
            id: self.ap_id,
            attributed_to: creator.ap_id,
            to: vec![public()],
            content: self.content,
            in_reply_to,
            published: Some(self.published),
            updated: self.updated,
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
        let parent = json.in_reply_to.dereference(context).await?;
        let (article_id, parent_id, depth) = match parent {
            DbArticleOrComment::Article(db_article) => (db_article.id, None, 0),
            DbArticleOrComment::Comment(db_comment) => (
                db_comment.article_id,
                Some(db_comment.id),
                db_comment.depth + 1,
            ),
        };
        let creator = json.attributed_to.dereference(context).await?;
        validate_comment_max_depth(depth)?;

        let form = DbCommentInsertForm {
            article_id,
            creator_id: creator.id,
            parent_id,
            ap_id: Some(json.id),
            local: false,
            deleted: false,
            published: json.published.unwrap_or_else(Utc::now),
            updated: json.updated,
            content: json.content,
            depth,
        };

        Ok(DbComment::create_or_update(form, context)?)
    }
}
