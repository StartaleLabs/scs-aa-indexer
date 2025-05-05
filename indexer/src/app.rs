use std::sync::Arc;
use crate::storage::Storage;
use crate::cache::Cache;

pub struct AppContext<S: Storage + Send + Sync + 'static, C: Cache + Send + Sync + 'static> {
    pub storage: Arc<S>,
    pub cache: Arc<C>,
}

impl<S: Storage + Send + Sync + 'static, C: Cache + Send + Sync + 'static> AppContext<S, C> {
    pub fn new(storage: Arc<S>, cache: Arc<C>) -> Self {
        Self { storage, cache }
    }
}