use crate::axum::http::listen;
use crate::{instance::new_instance, objects::post::DbPost, utils::generate_object_id};
use error::Error;
use tracing::log::LevelFilter;

mod activities;
mod axum;
mod error;
mod instance;
mod objects;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::builder()
        .filter_level(LevelFilter::Warn)
        .filter_module("activitypub_federation", LevelFilter::Info)
        .filter_module("fediwiki", LevelFilter::Info)
        .init();

    let alpha = new_instance("localhost:8001", "alpha".to_string()).await?;
    listen(&alpha)?;

    Ok(())
}
