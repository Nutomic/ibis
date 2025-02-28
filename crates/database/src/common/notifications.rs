use super::{
    article::{Article, Edit},
    comment::Comment,
    newtypes::{ArticleNotifId, ConflictId},
    user::Person,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiNotification {
    pub id: ArticleNotifId,
    pub creator: Person,
    pub article: Article,
    pub published: DateTime<Utc>,
    pub data: ApiNotificationData,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ApiNotificationData {
    ArticleCreated,
    EditConflict {
        conflict_id: ConflictId,
        summary: String,
    },
    Comment(Comment),
    Edit(Edit),
}
