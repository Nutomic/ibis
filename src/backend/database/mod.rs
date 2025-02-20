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
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::ops::DerefMut;

pub mod article;
pub mod comment;
pub mod conflict;
pub mod edit;
pub mod instance;
pub mod instance_stats;
pub mod notifications;
pub(crate) mod schema;
pub mod user;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct IbisContext {
    pub db_pool: DbPool,
    pub config: IbisConfig,
}

impl IbisContext {
    pub fn init(config: IbisConfig) -> BackendResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(&config.database.connection_url);
        let db_pool = Pool::builder()
            .max_size(config.database.pool_size)
            .build(manager)?;

        db_pool
            .get()?
            .run_pending_migrations(MIGRATIONS)
            .expect("run migrations");
        Ok(IbisContext { db_pool, config })
    }
}

pub fn read_jwt_secret(context: &IbisContext) -> BackendResult<String> {
    let mut conn = context.db_pool.get()?;
    Ok(jwt_secret::table
        .select(jwt_secret::dsl::secret)
        .first(conn.deref_mut())?)
}
