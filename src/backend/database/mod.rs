use crate::backend::database::article::DbArticle;
use diesel::PgConnection;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
pub type MyDataHandle = MyData;
use crate::backend::database::schema::jwt_secret;
use crate::backend::error::MyResult;
use diesel::{QueryDsl, RunQueryDsl};
use std::ops::DerefMut;

pub mod article;
pub mod conflict;
pub mod edit;
pub mod instance;
mod schema;
pub mod user;
pub mod version;

#[derive(Clone)]
pub struct MyData {
    pub db_connection: Arc<Mutex<PgConnection>>,
}

impl Deref for MyData {
    type Target = Arc<Mutex<PgConnection>>;

    fn deref(&self) -> &Self::Target {
        &self.db_connection
    }
}

pub fn read_jwt_secret(conn: &Mutex<PgConnection>) -> MyResult<String> {
    let mut conn = conn.lock().unwrap();
    Ok(jwt_secret::table
        .select(jwt_secret::dsl::secret)
        .first(conn.deref_mut())?)
}
