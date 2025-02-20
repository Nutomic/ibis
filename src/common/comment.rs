use super::{
    newtypes::{ArticleId, CommentId, PersonId},
    user::Person,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use {
    crate::backend::database::schema::comment,
    activitypub_federation::fetch::object_id::ObjectId,
    diesel::{Identifiable, Queryable, Selectable},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "ssr", diesel(table_name = comment, check_for_backend(diesel::pg::Pg), belongs_to(DbArticle, foreign_key = instance_id)))]
pub struct Comment {
    pub id: CommentId,
    pub creator_id: PersonId,
    pub article_id: ArticleId,
    pub parent_id: Option<CommentId>,
    pub content: String,
    pub depth: i32,
    #[cfg(feature = "ssr")]
    pub ap_id: ObjectId<Comment>,
    #[cfg(not(feature = "ssr"))]
    pub ap_id: String,
    pub local: bool,
    pub deleted: bool,
    pub published: DateTime<Utc>,
    pub updated: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct CommentView {
    pub comment: Comment,
    pub creator: Person,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateCommentParams {
    pub content: String,
    pub article_id: ArticleId,
    pub parent_id: Option<CommentId>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EditCommentParams {
    pub id: CommentId,
    pub content: Option<String>,
    pub deleted: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct DeleteCommentParams {
    pub id: CommentId,
}
