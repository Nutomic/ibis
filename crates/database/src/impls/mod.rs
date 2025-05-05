use crate::{config::IbisConfig, error::BackendResult};
use diesel::{
    PgConnection,
    QueryDsl,
    RunQueryDsl,
    define_sql_function,
    r2d2::{ConnectionManager, Pool},
    result::{DatabaseErrorKind, Error},
    sql_types,
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use ibis_database_schema::jwt_secret;
use reqwest::Client;
use std::{env::var, ops::DerefMut};

pub mod article;
pub mod comment;
pub mod conflict;
pub mod edit;
pub mod instance;
pub mod instance_stats;
pub mod notifications;
pub mod sent_activity;
pub mod user;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

#[derive(Clone)]
pub struct IbisContext {
    pub db_pool: DbPool,
    pub conf: IbisConfig,
    pub client: Client,
}

impl IbisContext {
    pub fn init(config: IbisConfig, ignore_env: bool) -> BackendResult<Self> {
        let database_url = config.database.connection_url.clone();
        let database_url = if ignore_env {
            database_url
        } else {
            var("DATABASE_URL").unwrap_or(database_url)
        };
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let db_pool = Pool::builder()
            .max_size(config.database.pool_size)
            .build(manager)?;

        db_pool
            .get()?
            .run_pending_migrations(MIGRATIONS)
            .expect("run migrations");
        let client = Client::builder().user_agent("ibis").build()?;

        Ok(IbisContext {
            db_pool,
            conf: config,
            client,
        })
    }
}

pub fn read_jwt_secret(context: &IbisContext) -> BackendResult<String> {
    let mut conn = context.db_pool.get()?;
    Ok(jwt_secret::table
        .select(jwt_secret::dsl::secret)
        .first(conn.deref_mut())?)
}

define_sql_function!(fn lower(x: sql_types::Text) -> sql_types::Text);

define_sql_function!(fn coalesce<T: sql_types::SqlType + sql_types::SingleValue>(x: sql_types::Nullable<T>, y: T) -> T);

/// Need to handle conflicts manually to avoid duplicate notifications
fn is_conflict<T>(val: &Result<T, Error>) -> bool {
    matches!(
        val.as_ref().err(),
        Some(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _))
    )
}
