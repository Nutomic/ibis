use crate::database::edit::DbEdit;

use crate::database::schema::{article, edit};
use crate::error::MyResult;
use crate::federation::objects::edits_collection::DbEditCollection;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use diesel::pg::PgConnection;

use diesel::ExpressionMethods;
use diesel::{
    insert_into, AsChangeset, BoolExpressionMethods, Identifiable, Insertable,
    PgTextExpressionMethods, QueryDsl, Queryable, RunQueryDsl, Selectable,
};
use serde::{Deserialize, Serialize};

use crate::database::version::EditVersion;
use std::ops::DerefMut;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg), belongs_to(DbInstance, foreign_key = instance_id))]
pub struct DbArticle {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub ap_id: ObjectId<DbArticle>,
    pub instance_id: i32,
    pub local: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct ArticleView {
    pub article: DbArticle,
    pub latest_version: EditVersion,
    pub edits: Vec<DbEdit>,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct DbArticleForm {
    pub title: String,
    pub text: String,
    pub ap_id: ObjectId<DbArticle>,
    pub instance_id: i32,
    pub local: bool,
}

impl DbArticle {
    pub fn edits_id(&self) -> MyResult<CollectionId<DbEditCollection>> {
        Ok(CollectionId::parse(&format!("{}/edits", self.ap_id))?)
    }

    pub fn create(form: &DbArticleForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(article::table)
            .values(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn create_or_update(form: &DbArticleForm, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(insert_into(article::table)
            .values(form)
            .on_conflict(article::dsl::ap_id)
            .do_update()
            .set(form)
            .get_result(conn.deref_mut())?)
    }

    pub fn update_text(id: i32, text: &str, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(diesel::update(article::dsl::article.find(id))
            .set(article::dsl::text.eq(text))
            .get_result::<Self>(conn.deref_mut())?)
    }

    pub fn read(id: i32, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(article::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn read_view(id: i32, conn: &Mutex<PgConnection>) -> MyResult<ArticleView> {
        let article: DbArticle = {
            let mut conn = conn.lock().unwrap();
            article::table.find(id).get_result(conn.deref_mut())?
        };
        let latest_version = article.latest_edit_version(conn)?;
        let edits: Vec<DbEdit> = DbEdit::read_for_article(&article, conn)?;
        Ok(ArticleView {
            article,
            edits,
            latest_version,
        })
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbArticle>,
        conn: &Mutex<PgConnection>,
    ) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(article::table
            .filter(article::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_local_title(title: &str, conn: &Mutex<PgConnection>) -> MyResult<Self> {
        let mut conn = conn.lock().unwrap();
        Ok(article::table
            .filter(article::dsl::title.eq(title))
            .filter(article::dsl::local.eq(true))
            .get_result(conn.deref_mut())?)
    }

    pub fn read_all_local(conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        let mut conn = conn.lock().unwrap();
        Ok(article::table
            .filter(article::dsl::local.eq(true))
            .get_results(conn.deref_mut())?)
    }

    pub fn search(query: &str, conn: &Mutex<PgConnection>) -> MyResult<Vec<Self>> {
        let mut conn = conn.lock().unwrap();
        let replaced = query
            .replace('%', "\\%")
            .replace('_', "\\_")
            .replace(' ', "%");
        Ok(article::table
            .filter(
                article::dsl::title
                    .ilike(&replaced)
                    .or(article::dsl::text.ilike(&replaced)),
            )
            .get_results(conn.deref_mut())?)
    }

    pub fn latest_edit_version(&self, conn: &Mutex<PgConnection>) -> MyResult<EditVersion> {
        let mut conn = conn.lock().unwrap();
        let latest_version: Option<EditVersion> = edit::table
            .filter(edit::dsl::article_id.eq(self.id))
            .order_by(edit::dsl::id.desc())
            .limit(1)
            .select(edit::dsl::hash)
            .get_result(conn.deref_mut())
            .ok();
        match latest_version {
            Some(latest_version) => Ok(latest_version),
            None => Ok(EditVersion::default()),
        }
    }
}
