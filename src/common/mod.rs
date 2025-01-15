pub mod article;
pub mod comment;
pub mod instance;
pub mod newtypes;
pub mod user;
pub mod utils;
pub mod validation;

use article::{ApiConflict, DbArticle};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use url::Url;

pub const MAIN_PAGE_NAME: &str = "Main_Page";

pub static AUTH_COOKIE: &str = "auth";

#[derive(Clone, Debug)]
pub struct Auth(pub Option<String>);

#[derive(Deserialize, Serialize, Debug)]
pub struct SuccessResponse {
    success: bool,
}

impl Default for SuccessResponse {
    fn default() -> Self {
        Self { success: true }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResolveObject {
    pub id: Url,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Notification {
    EditConflict(ApiConflict),
    ArticleApprovalRequired(DbArticle),
}

impl Notification {
    pub fn published(&self) -> &DateTime<Utc> {
        match self {
            Notification::EditConflict(api_conflict) => &api_conflict.published,
            Notification::ArticleApprovalRequired(db_article) => &db_article.published,
        }
    }
}
