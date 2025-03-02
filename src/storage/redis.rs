use redis::AsyncCommands;
use std::sync::Arc;

/// **Redis Storage for Event Caching**
pub struct RedisStorage {
    client: redis::Client,
}

impl RedisStorage {
    pub fn new(redis_url: &str) -> Arc<Self> {
        let client = redis::Client::open(redis_url).expect("Failed to connect to Redis");
        Arc::new(Self { client })
    }

    /// **Cache an Event in Redis**
    pub async fn cache_event(&self, event_name: &str, event_data: &str) {
        let mut conn = self.client.get_async_connection().await.unwrap();
        let _ : () = conn.set(event_name, event_data).await.unwrap();
        println!("✅ Cached event in Redis: {}", event_name);
    }
}
