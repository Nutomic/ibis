use crate::{
    backend::{
        database::IbisContext,
        federation::VerifyUrlData,
        utils::{config::IbisConfig, error::BackendResult, generate_activity_id},
    },
    common::instance::DbInstance,
};
use activitypub_federation::config::FederationConfig;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::info;
use server::{setup::setup, start_server};
use std::{net::SocketAddr, thread};
use tokio::sync::oneshot;
use utils::scheduled_tasks;

pub mod api;
pub mod database;
pub mod federation;
mod server;
pub mod utils;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn start(
    config: IbisConfig,
    override_hostname: Option<SocketAddr>,
    notify_start: Option<oneshot::Sender<()>>,
) -> BackendResult<()> {
    let manager = ConnectionManager::<PgConnection>::new(&config.database.connection_url);
    let db_pool = Pool::builder()
        .max_size(config.database.pool_size)
        .build(manager)?;

    db_pool
        .get()?
        .run_pending_migrations(MIGRATIONS)
        .expect("run migrations");
    let context = IbisContext { db_pool, config };
    let data = FederationConfig::builder()
        .domain(context.config.federation.domain.clone())
        .url_verifier(Box::new(VerifyUrlData(context.config.clone())))
        .app_data(context)
        .http_fetch_limit(1000)
        .debug(cfg!(debug_assertions))
        .build()
        .await?;

    if DbInstance::read_local(&data).is_err() {
        info!("Running setup for new instance");
        setup(&data.to_request_data()).await?;
    }

    let db_pool = data.db_pool.clone();
    thread::spawn(move || {
        scheduled_tasks::start(db_pool);
    });

    start_server(data, override_hostname, notify_start).await?;

    Ok(())
}
