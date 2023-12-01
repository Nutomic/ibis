use crate::api::api_routes;
use crate::database::MyData;
use crate::error::MyResult;
use crate::federation::routes::federation_routes;
use crate::utils::generate_activity_id;
use activitypub_federation::config::{FederationConfig, FederationMiddleware};
use axum::{Router, Server};
use diesel::Connection;
use diesel::PgConnection;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use federation::create_fake_db;
use std::net::ToSocketAddrs;
use std::sync::{Arc, Mutex};
use tracing::info;

pub mod api;
pub mod database;
pub mod error;
pub mod federation;
mod utils;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub async fn start(hostname: &str, database_url: &str) -> MyResult<()> {
    let fake_db = create_fake_db(hostname).await?;

    let db_connection = Arc::new(Mutex::new(PgConnection::establish(database_url)?));
    db_connection
        .lock()
        .unwrap()
        .run_pending_migrations(MIGRATIONS)
        .unwrap();

    let data = MyData {
        db_connection,
        fake_db,
    };
    let config = FederationConfig::builder()
        .domain(hostname)
        .app_data(data)
        .debug(true)
        .build()
        .await?;

    info!("Listening with axum on {hostname}");
    let config = config.clone();
    let app = Router::new()
        .nest("", federation_routes())
        .nest("/api/v1", api_routes())
        .layer(FederationMiddleware::new(config));

    let addr = hostname
        .to_socket_addrs()?
        .next()
        .expect("Failed to lookup domain name");
    let server = Server::bind(&addr).serve(app.into_make_service());

    tokio::spawn(server);

    Ok(())
}
