pub mod kafka;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait Storage {
    async fn store<T: Serialize + Send + Sync> (&self, event: &T, event_name: &str);
}