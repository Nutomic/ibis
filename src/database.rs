use crate::federation::objects::instance::DbInstance;
use crate::federation::objects::{article::DbArticle, person::DbUser};

use std::sync::{Arc, Mutex};

pub type DatabaseHandle = Arc<Database>;

/// Our "database" which contains all known posts and users (local and federated)
pub struct Database {
    pub instances: Mutex<Vec<DbInstance>>,
    pub users: Mutex<Vec<DbUser>>,
    pub articles: Mutex<Vec<DbArticle>>,
}

impl Database {
    pub fn local_instance(&self) -> DbInstance {
        let lock = self.instances.lock().unwrap();
        lock.iter().find(|i| i.local).unwrap().clone()
    }
}
