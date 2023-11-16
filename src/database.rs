use crate::federation::objects::article::DbArticle;
use crate::federation::objects::instance::DbInstance;

use std::sync::{Arc, Mutex};

pub type DatabaseHandle = Arc<Database>;

pub struct Database {
    pub instances: Mutex<Vec<DbInstance>>,
    pub articles: Mutex<Vec<DbArticle>>,
}

impl Database {
    pub fn local_instance(&self) -> DbInstance {
        let lock = self.instances.lock().unwrap();
        lock.iter().find(|i| i.local).unwrap().clone()
    }
}
