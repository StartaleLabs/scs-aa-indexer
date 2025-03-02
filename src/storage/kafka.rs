use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use async_trait::async_trait;
use serde_json::to_string;
use serde::Serialize;
use std::time::Duration;
use crate::storage::Storage;

/// **Kafka Storage Implementation**
pub struct KafkaStorage {
    producer: FutureProducer,
    topic: String,
}

impl KafkaStorage {
    pub fn new(broker: &str, topic: &str) -> Self {
        let producer = ClientConfig::new()
            .set("bootstrap.servers", broker)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Failed to create Kafka producer");

        Self {
            producer,
            topic: topic.to_string(),
        }
    }
}

#[async_trait]
impl Storage for KafkaStorage {
    async fn store<T: Serialize + Send + Sync>(&self, event: &T, event_name: &str) {
        let payload = to_string(event).expect("Failed to serialize event");

        let record = FutureRecord::to(&self.topic)
            .key(event_name)
            .payload(&payload);

        match self.producer.send(record, Duration::from_secs(2)).await {
            Ok(_) => println!("✅ Event `{}` published to Kafka", event_name),
            Err(e) => eprintln!("❌ Failed to publish event `{}`: {:?}", event_name, e),
        }
    }
}
