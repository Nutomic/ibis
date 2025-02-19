use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{
    article::{ApiConflict, DbArticle},
    comment::CommentViewWithArticle,
    newtypes::{ArticleNotifId, CommentId},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Notification {
    // TODO: this should only return conflict id and article name
    EditConflict(ApiConflict),
    ArticleApprovalRequired(DbArticle),
    Reply(CommentViewWithArticle),
    ArticleNotification(ArticleNotificationView),
}

impl Notification {
    pub fn published(&self) -> &DateTime<Utc> {
        use Notification::*;
        match self {
            EditConflict(c) => &c.published,
            ArticleApprovalRequired(a) => &a.published,
            Reply(c) => &c.comment.published,
            ArticleNotification(n) => &n.published,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArticleNotificationView {
    pub article: DbArticle,
    pub id: ArticleNotifId,
    pub kind: ArticleNotificationKind,
    pub published: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct MarkAsReadParams {
    pub id: CommentId,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ArticleNotificationKind {
    Comment,
    Edit,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ArticleNotifMarkAsReadParams {
    pub id: ArticleNotifId,
}
