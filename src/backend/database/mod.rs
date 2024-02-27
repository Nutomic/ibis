use crate::backend::config::IbisConfig;
use crate::backend::database::schema::jwt_secret;
use crate::backend::error::MyResult;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use diesel::{QueryDsl, RunQueryDsl};

use std::ops::DerefMut;

pub mod article;
pub mod conflict;
pub mod edit;
pub mod instance;
pub(crate) mod schema;
pub mod user;

#[derive(Clone)]
pub struct IbisData {
    pub db_pool: Pool<ConnectionManager<PgConnection>>,
    pub config: IbisConfig,
}

pub fn read_jwt_secret(data: &IbisData) -> MyResult<String> {
    let mut conn = data.db_pool.get()?;
    Ok(jwt_secret::table
        .select(jwt_secret::dsl::secret)
        .first(conn.deref_mut())?)
}
