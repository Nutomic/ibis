use crate::database::{Database, DatabaseHandle};
use crate::error::Error;
use crate::federation::objects::instance::DbInstance;

use activitypub_federation::config::{FederationConfig, UrlVerifier};
use activitypub_federation::http_signatures::generate_actor_keypair;
use async_trait::async_trait;
use chrono::Local;
use std::sync::{Arc, Mutex};
use url::Url;

pub mod activities;
pub mod objects;
pub mod routes;

pub async fn federation_config(hostname: &str) -> Result<FederationConfig<DatabaseHandle>, Error> {
    let ap_id = Url::parse(&format!("http://{}", hostname))?.into();
    let inbox = Url::parse(&format!("http://{}/inbox", hostname))?;
    let keypair = generate_actor_keypair()?;
    let local_instance = DbInstance {
        ap_id,
        inbox,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Local::now().naive_local(),
        followers: vec![],
        follows: vec![],
        local: true,
    };
    let database = Arc::new(Database {
        instances: Mutex::new(vec![local_instance]),
        users: Mutex::new(vec![]),
        articles: Mutex::new(vec![]),
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
