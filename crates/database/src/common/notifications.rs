use super::{
    article::{Article, Edit},
    comment::Comment,
    newtypes::{ArticleNotifId, CommentId},
    user::Person,
};
use crate::impls::conflict::Conflict;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApiNotification {
    // TODO: this should only return conflict id and article name
    EditConflict(Conflict, Article),
    ArticleApprovalRequired(Article),
    Comment(ArticleNotifId, Comment, Person, Article),
    Edit(ArticleNotifId, Edit, Person, Article),
}

impl ApiNotification {
    pub fn published(&self) -> &DateTime<Utc> {
        use ApiNotification::*;
        match self {
            EditConflict(c, _) => &c.published,
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
