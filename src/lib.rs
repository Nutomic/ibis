use crate::utils::generate_object_id;
use tracing::log::LevelFilter;

use activitypub_federation::config::FederationMiddleware;
use axum::{
    routing::{get, post},
    Router, Server,
};

use crate::api::api_get_article;
use crate::error::MyResult;
use crate::federation::routes::http_get_user;
use crate::federation::routes::http_post_user_inbox;
use federation::federation_config;
use std::net::ToSocketAddrs;
use tracing::info;

mod api;
mod database;
pub mod error;
pub mod federation;
mod utils;

pub async fn start(hostname: &str) -> MyResult<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Info)
        .filter_module("fediwiki", LevelFilter::Info)
        .init();

    let config = federation_config(hostname).await?;

    info!("Listening with axum on {hostname}");
    let config = config.clone();
    let app = Router::new()
        // federation routes
        .route("/:user/inbox", post(http_post_user_inbox))
        .route("/:user", get(http_get_user))
        // api routes
        .route("/api/v1/article/:title", get(api_get_article))
        .layer(FederationMiddleware::new(config));

    let addr = hostname
        .to_socket_addrs()?
        .next()
        .expect("Failed to lookup domain name");
    let server = Server::bind(&addr).serve(app.into_make_service());

    tokio::spawn(server);

    Ok(())
}
