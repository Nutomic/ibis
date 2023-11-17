use crate::database::{Database, DatabaseHandle};
use crate::error::Error;
use crate::federation::objects::instance::DbInstance;
use activitypub_federation::config::FederationConfig;
use activitypub_federation::http_signatures::generate_actor_keypair;
use chrono::Local;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;

pub mod activities;
pub mod objects;
pub mod routes;

pub async fn federation_config(hostname: &str) -> Result<FederationConfig<DatabaseHandle>, Error> {
    let ap_id = Url::parse(&format!("http://{}", hostname))?.into();
    let articles_id = Url::parse(&format!("http://{}/articles", hostname))?.into();
    let inbox = Url::parse(&format!("http://{}/inbox", hostname))?;
    let keypair = generate_actor_keypair()?;
    let local_instance = DbInstance {
        ap_id,
        articles_id,
        inbox,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Local::now().into(),
        followers: vec![],
        follows: vec![],
        local: true,
    };
    let database = Arc::new(Database {
        instances: Mutex::new(HashMap::from([(
            local_instance.ap_id.inner().clone(),
            local_instance,
        )])),
        articles: Mutex::new(HashMap::new()),
    });
    let config = FederationConfig::builder()
        .domain(hostname)
        .app_data(database)
        .debug(true)
        .build()
        .await?;
    Ok(config)
}
