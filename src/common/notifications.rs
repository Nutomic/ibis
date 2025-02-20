use super::{
    article::{ApiConflict, DbArticle, DbEdit},
    comment::DbComment,
    newtypes::{ArticleNotifId, CommentId},
    user::DbPerson,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApiNotification {
    // TODO: this should only return conflict id and article name
    EditConflict(ApiConflict),
    ArticleApprovalRequired(DbArticle),
    Comment(ArticleNotifId, DbComment, DbPerson, DbArticle),
    Edit(ArticleNotifId, DbEdit, DbPerson, DbArticle),
}

impl ApiNotification {
    pub fn published(&self) -> &DateTime<Utc> {
        use ApiNotification::*;
        match self {
            EditConflict(c) => &c.published,
            ArticleApprovalRequired(a) => &a.published,
            Comment(_, c, _, _) => &c.published,
            Edit(_, e, _, _) => &e.published,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MarkAsReadParams {
    pub id: CommentId,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArticleNotifMarkAsReadParams {
    pub id: ArticleNotifId,
}
