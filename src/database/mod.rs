use crate::database::article::DbArticle;

use diesel::PgConnection;
use std::ops::Deref;
use std::sync::{Arc, Mutex};

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

pub type MyDataHandle = MyData;
