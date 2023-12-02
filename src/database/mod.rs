use crate::api::ApiConflict;
use crate::database::article::DbArticle;
use crate::database::edit::DbEdit;
use crate::error::MyResult;
use crate::federation::activities::submit_article_update;
use crate::utils::generate_article_version;
use activitypub_federation::config::Data;
use activitypub_federation::fetch::object_id::ObjectId;
use diesel::PgConnection;
use diffy::{apply, merge, Patch};
use edit::EditVersion;
use instance::DbInstance;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use url::Url;

pub mod article;
pub mod edit;
pub mod instance;
mod schema;

#[derive(Clone)]
pub struct MyData {
    pub db_connection: Arc<Mutex<PgConnection>>,
    pub fake_db: Arc<FakeDatabase>,
}

impl Deref for MyData {
    type Target = Arc<FakeDatabase>;

    fn deref(&self) -> &Self::Target {
        &self.fake_db
    }
}
pub type MyDataHandle = MyData;

pub struct FakeDatabase {
    pub conflicts: Mutex<Vec<DbConflict>>,
}

#[derive(Clone, Debug)]
pub struct DbConflict {
    pub id: i32,
    pub diff: String,
    pub article_id: ObjectId<DbArticle>,
    pub previous_version: EditVersion,
}

impl DbConflict {
    pub async fn to_api_conflict(
        &self,
        data: &Data<MyDataHandle>,
    ) -> MyResult<Option<ApiConflict>> {
        let original_article =
            DbArticle::read_from_ap_id(&self.article_id.clone(), &data.db_connection)?;

        // create common ancestor version
        let edits = DbEdit::for_article(&original_article, &data.db_connection)?;
        let ancestor = generate_article_version(&edits, &self.previous_version)?;
        dbg!(&ancestor, &self.previous_version);

        dbg!(&self.diff);
        let patch = Patch::from_str(&self.diff)?;
        // apply self.diff to ancestor to get `ours`
        let ours = dbg!(apply(&ancestor, &patch))?;
        match merge(&ancestor, &ours, &original_article.text) {
            Ok(new_text) => {
                // patch applies cleanly so we are done
                // federate the change
                submit_article_update(
                    data,
                    new_text,
                    self.previous_version.clone(),
                    &original_article,
                )
                .await?;
                // remove conflict from db
                let mut lock = data.conflicts.lock().unwrap();
                lock.retain(|c| c.id != self.id);
                Ok(None)
            }
            Err(three_way_merge) => {
                // there is a merge conflict, user needs to do three-way-merge
                Ok(Some(ApiConflict {
                    id: self.id,
                    three_way_merge,
                    article_id: original_article.ap_id.clone(),
                    previous_version: original_article.latest_edit_version(&data.db_connection)?,
                }))
            }
        }
    }
}
