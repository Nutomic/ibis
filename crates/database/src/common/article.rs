use super::{
    comment::CommentView,
    instance::Instance,
    newtypes::{ArticleId, ConflictId, EditId, InstanceId, PersonId},
    user::Person,
};
use crate::{DbUrl, common::utils::extract_domain};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[cfg(feature = "ssr")]
use {
    diesel::{Identifiable, Queryable, Selectable},
    ibis_database_schema::{article, conflict, edit},
    sha2::{Digest, Sha256},
};

/// A local only object which represents a merge conflict. It is created
/// when a local user edit conflicts with another concurrent edit.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "ssr", diesel(table_name = conflict, check_for_backend(diesel::pg::Pg), belongs_to(DbArticle, foreign_key = article_id)))]
pub struct Conflict {
    pub id: ConflictId,
    pub hash: EditVersion,
    #[serde(skip)]
    pub diff: String,
    pub summary: String,
    pub creator_id: PersonId,
    pub article_id: ArticleId,
    pub previous_version_id: EditVersion,
    pub published: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(table_name = article, check_for_backend(diesel::pg::Pg)))]
pub struct ArticleView {
    pub article: Article,
    pub instance: Instance,
    pub comments: Vec<CommentView>,
    pub latest_version: EditVersion,
    pub following: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable, Identifiable))]
#[cfg_attr(feature = "ssr", diesel(table_name = article, check_for_backend(diesel::pg::Pg), belongs_to(DbInstance, foreign_key = instance_id)))]
pub struct Article {
    pub id: ArticleId,
    pub title: String,
    pub text: String,
    pub ap_id: DbUrl,
    pub instance_id: InstanceId,
    pub local: bool,
    pub protected: bool,
    pub published: DateTime<Utc>,
    pub removed: bool,
    pub updated: DateTime<Utc>,
    pub pending: bool,
}

impl Article {
    pub fn title(&self) -> String {
        if self.local {
            self.title.clone()
        } else {
            format!("{}@{}", self.title, extract_domain(self.ap_id.inner()))
        }
    }
}
/// Represents a single change to the article.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable, Selectable))]
#[cfg_attr(feature = "ssr", diesel(table_name = edit, check_for_backend(diesel::pg::Pg)))]
pub struct Edit {
    // TODO: we could use hash as primary key, but that gives errors on forking because
    //       the same edit is used for multiple articles
    pub id: EditId,
    #[serde(skip)]
    pub creator_id: PersonId,
    /// UUID built from sha224 hash of diff
    pub hash: EditVersion,
    pub ap_id: DbUrl,
    pub diff: String,
    pub summary: String,
    pub article_id: ArticleId,
    /// First edit of an article always has `EditVersion::default()` here
    pub previous_version_id: EditVersion,
    pub published: DateTime<Utc>,
    pub pending: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(Queryable))]
#[cfg_attr(feature = "ssr", diesel(check_for_backend(diesel::pg::Pg)))]
pub struct EditView {
    pub edit: Edit,
    pub article: Article,
    pub creator: Person,
}

/// The version hash of a specific edit. Generated by taking an SHA256 hash of the diff
/// and using the first 16 bytes so that it fits into UUID.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ssr", derive(diesel_derive_newtype::DieselNewType))]
pub struct EditVersion(pub Uuid);

#[cfg(feature = "ssr")]
impl EditVersion {
    pub fn new(diff: &str) -> Self {
        let mut sha256 = Sha256::new();
        sha256.update(diff);
        let hash_bytes = sha256.finalize();
        let uuid =
            Uuid::from_slice(&hash_bytes.as_slice()[..16]).expect("hash is correct size for uuid");
        EditVersion(uuid)
    }

    pub fn hash(&self) -> String {
        hex::encode(self.0.into_bytes())
    }
}

#[cfg(feature = "ssr")]
impl Default for EditVersion {
    fn default() -> Self {
        EditVersion::new("")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ApiConflict {
    pub id: ConflictId,
    pub hash: EditVersion,
    pub three_way_merge: String,
    pub summary: String,
    pub article: Article,
    pub previous_version_id: EditVersion,
    pub published: DateTime<Utc>,
}

pub fn can_edit_article(article: &Article, is_admin: bool) -> Result<(), anyhow::Error> {
    if article.protected && !(article.local && is_admin) {
        return Err(anyhow!(
            "Article is protected, only admins on origin instance can edit".to_string()
        ));
    }
    Ok(())
}

#[test]
fn test_edit_versions() {
    let default = EditVersion::default();
    assert_eq!("e3b0c44298fc1c149afbf4c8996fb924", default.hash());

    let version = EditVersion::new("test");
    assert_eq!("9f86d081884c7d659a2feaa0c55ad015", version.hash());
}
