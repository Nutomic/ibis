use crate::backend::{
    database::schema::jwt_secret,
    utils::{config::IbisConfig, error::BackendResult},
};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
    QueryDsl,
    RunQueryDsl,
};
use std::ops::DerefMut;

pub mod article;
pub mod comment;
pub mod conflict;
pub mod edit;
pub mod instance;
pub mod instance_stats;
pub(crate) mod schema;
pub mod user;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct IbisContext {
    pub db_pool: DbPool,
    pub config: IbisConfig,
}

pub fn read_jwt_secret(context: &IbisContext) -> BackendResult<String> {
    let mut conn = context.db_pool.get()?;
    Ok(jwt_secret::table
        .select(jwt_secret::dsl::secret)
        .first(conn.deref_mut())?)
}
