use crate::database::FakeDatabase;
use crate::error::Error;
use crate::federation::objects::instance::DbInstance;
use activitypub_federation::fetch::collection_id::CollectionId;
use activitypub_federation::http_signatures::generate_actor_keypair;
use chrono::Local;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;

pub mod activities;
pub mod objects;
pub mod routes;

pub async fn create_fake_db(hostname: &str) -> Result<Arc<FakeDatabase>, Error> {
    let ap_id = Url::parse(&format!("http://{}", hostname))?;
    let articles_id = CollectionId::parse(&format!("http://{}/all_articles", hostname))?;
    let inbox = Url::parse(&format!("http://{}/inbox", hostname))?;
    let keypair = generate_actor_keypair()?;
    let local_instance = DbInstance {
        ap_id: ap_id.into(),
        articles_id,
        inbox,
        public_key: keypair.public_key,
        private_key: Some(keypair.private_key),
        last_refreshed_at: Local::now().into(),
        followers: vec![],
        follows: vec![],
        local: true,
    };
    let fake_db = Arc::new(FakeDatabase {
        instances: Mutex::new(HashMap::from([(
            local_instance.ap_id.inner().clone(),
            local_instance,
        )])),
        conflicts: Mutex::new(vec![]),
    });
    Ok(fake_db)
}
