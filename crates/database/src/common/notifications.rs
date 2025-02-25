use super::{
    article::{Article, Conflict, Edit},
    comment::Comment,
    newtypes::ArticleNotifId,
    user::Person,
};
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
