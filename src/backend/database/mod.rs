use crate::backend::database::schema::jwt_secret;
use crate::backend::error::MyResult;
use crate::config::IbisConfig;
use diesel::PgConnection;
use diesel::{QueryDsl, RunQueryDsl};
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};

pub mod article;
pub mod conflict;
pub mod edit;
pub mod instance;
pub(crate) mod schema;
pub mod user;
pub mod version;

#[derive(Clone)]
pub struct IbisData {
    pub db_connection: Arc<Mutex<PgConnection>>,
    pub config: IbisConfig,
}

impl Deref for IbisData {
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
