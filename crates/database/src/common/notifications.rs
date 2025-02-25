use super::{
    article::{ApiConflict, Article, Edit},
    comment::Comment,
    newtypes::{ArticleNotifId, CommentId},
    user::Person,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApiNotification {
    // TODO: this should only return conflict id and article name
    EditConflict(ApiConflict),
    ArticleApprovalRequired(Article),
    Comment(ArticleNotifId, Comment, Person, Article),
    Edit(ArticleNotifId, Edit, Person, Article),
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
