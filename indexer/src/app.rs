use std::sync::Arc;
use crate::cache::redis::RedisCoordinator;
use crate::storage::Storage;

pub struct AppContext<S: Storage> {
    pub storage: Arc<S>,
    pub redis: Arc<UserOpRedisCoordinator>,
}

impl<S: Storage> AppContext<S> {
    pub fn new(storage: Arc<S>, redis: Arc<UserOpRedisCoordinator>) -> Self {
        Self { storage, redis }
    }
}