use crate::{instance::federation_config, utils::generate_object_id};
use error::Error;
use tracing::log::LevelFilter;

use activitypub_federation::config::FederationMiddleware;
use axum::{
    routing::{get, post},
    Router, Server,
};

use crate::federation::routes::http_get_user;
use crate::federation::routes::http_post_user_inbox;
use std::net::ToSocketAddrs;
use tracing::info;

mod error;
mod federation;
mod instance;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Info)
        .filter_module("fediwiki", LevelFilter::Info)
        .init();

    let config = federation_config("localhost:8001", "alpha".to_string()).await?;

    let hostname = config.domain();
    info!("Listening with axum on {hostname}");
    let config = config.clone();
    let app = Router::new()
        .route("/:user/inbox", post(http_post_user_inbox))
        .route("/:user", get(http_get_user))
        .layer(FederationMiddleware::new(config));

    let addr = hostname
        .to_socket_addrs()?
        .next()
        .expect("Failed to lookup domain name");
    let server = Server::bind(&addr).serve(app.into_make_service());

    tokio::spawn(server);
    Ok(())
}
