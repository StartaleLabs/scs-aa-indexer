use std::sync::Arc;

use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::ClientConfig;
use rdkafka::message::Message;
use crate::{
    consumer::kakfa_message::UserOpMessage,
    storage::Storage
};

pub fn start_kafka_consumer<S: Storage + Send + Sync + 'static>(
    brokers: &str,
    topic: &str,
    group_id: &str,
    app: Arc<AppContext<S>>,
) {
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", brokers)
        .set("auto.offset.reset", "earliest")
        .set("enable.partition.eof", "false")
        .set("debug", "all")
        .create()
        .expect("Failed to create Kafka consumer");

    consumer.subscribe(&[topic]).expect("Failed to subscribe to topic");
    println!("🟢 Kafka consumer subscribed to topic: {}", topic);

    tokio::spawn(async move {
        loop {
            match consumer.recv().await {
                Ok(m) => {
                    if let Some(payload) = m.payload_view::<str>().and_then(Result::ok) {
                        tracing::info!("📥 Received message: {:?}", payload);
                        match serde_json::from_str::<UserOpMessage>(payload) {
                            Ok(event) => {
                                if let Err(e) = app.storage.upsert_user_op_message(event).await {
                                    tracing::error!("❌ Failed to upsert UserOpMessage into Timescale: {:?}", e);
                                }
                            }
                            Err(e) => tracing::error!("❌ Failed to deserialize UserOpMessage: {:?}", e),
                        }
                    } else {
                        tracing::error!("❌ Failed to get payload from message");
                    }
                }
                Err(e) => tracing::error!("❌ Kafka error: {:?}", e),
            }
        }
    });
}
