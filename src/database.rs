use crate::api::Conflict;
use crate::federation::objects::article::DbArticle;

use crate::federation::objects::instance::DbInstance;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use url::Url;

pub type DatabaseHandle = Arc<Database>;

pub struct Database {
    pub instances: Mutex<HashMap<Url, DbInstance>>,
    pub articles: Mutex<HashMap<Url, DbArticle>>,
    pub conflicts: Mutex<Vec<Conflict>>,
}

impl Database {
    pub fn local_instance(&self) -> DbInstance {
        let lock = self.instances.lock().unwrap();
        lock.iter().find(|i| i.1.local).unwrap().1.clone()
    }
}
