use super::{
    article::{Article, Conflict, Edit},
    comment::Comment,
    newtypes::ArticleNotifId,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApiNotificationData {
    ArticleCreated,
    // TODO: this should only return conflict id and article name
    EditConflict(Conflict),
    Comment(Comment),
    Edit(Edit),
}
