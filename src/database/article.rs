use crate::database::edit::EditVersion;
use crate::database::schema::article;
use crate::error::MyResult;
use crate::federation::objects::edits_collection::DbEditCollection;
use crate::federation::objects::instance::DbInstance;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::fetch::object_id::ObjectId;
use diesel::pg::PgConnection;
use diesel::ExpressionMethods;
use diesel::{
    insert_into, AsChangeset, Identifiable, Insertable, QueryDsl, Queryable, RunQueryDsl,
    Selectable,
};
use serde::{Deserialize, Serialize};
use std::ops::DerefMut;
use std::sync::Mutex;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Queryable, Selectable, Identifiable)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct DbArticle {
    pub id: i32,
    pub title: String,
    pub text: String,
    pub ap_id: ObjectId<DbArticle>,
    pub instance_id: ObjectId<DbInstance>,
    /// List of all edits which make up this article, oldest first.
    // TODO
    //pub edits: Vec<DbEdit>,
    pub latest_version: EditVersion,
    pub local: bool,
}

#[derive(Debug, Clone, Insertable, AsChangeset)]
#[diesel(table_name = article, check_for_backend(diesel::pg::Pg))]
pub struct DbArticleForm {
    pub title: String,
    pub text: String,
    pub ap_id: ObjectId<DbArticle>,
    // TODO: change to foreign key
    pub instance_id: ObjectId<DbInstance>,
    // TODO: instead of this we can use latest entry in edits table
    pub latest_version: String,
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

    pub fn read(id: i32, conn: &Mutex<PgConnection>) -> MyResult<DbArticle> {
        let mut conn = conn.lock().unwrap();
        Ok(article::table.find(id).get_result(conn.deref_mut())?)
    }

    pub fn read_from_ap_id(
        ap_id: &ObjectId<DbArticle>,
        conn: &Mutex<PgConnection>,
    ) -> MyResult<DbArticle> {
        let mut conn = conn.lock().unwrap();
        Ok(article::table
            .filter(article::dsl::ap_id.eq(ap_id))
            .get_result(conn.deref_mut())?)
    }
}
