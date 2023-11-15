use crate::database::{Database, DatabaseHandle};
use crate::error::Error;
use activitypub_federation::config::{FederationConfig, UrlVerifier};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use url::Url;

pub mod activities;
pub mod objects;
pub mod routes;

pub async fn federation_config(hostname: &str) -> Result<FederationConfig<DatabaseHandle>, Error> {
    let database = Arc::new(Database {
        instances: Mutex::new(vec![]),
        users: Mutex::new(vec![]),
        posts: Mutex::new(vec![]),
    });
    let config = FederationConfig::builder()
        .domain(hostname)
        .app_data(database)
        .debug(true)
        .build()
        .await?;
    Ok(config)
}

/// Use this to store your federation blocklist, or a database connection needed to retrieve it.
#[derive(Clone)]
struct MyUrlVerifier();

#[async_trait]
impl UrlVerifier for MyUrlVerifier {
    async fn verify(&self, url: &Url) -> Result<(), &'static str> {
        if url.domain() == Some("malicious.com") {
            Err("malicious domain")
        } else {
            Ok(())
        }
    }
}
