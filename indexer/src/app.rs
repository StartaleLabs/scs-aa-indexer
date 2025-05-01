use std::sync::Arc;
use crate::cache::redis::RedisCoordinator;
use crate::storage::Storage;

pub struct AppContext<S: Storage> {
    pub storage: Arc<S>,
    pub cache: Arc<RedisCoordinator>,
}

impl<S: Storage> AppContext<S> {
    pub fn new(storage: Arc<S>, cache: Arc<RedisCoordinator>) -> Self {
        Self { storage, cache }
    }
}