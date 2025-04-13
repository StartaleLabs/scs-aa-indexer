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
    db: Arc<S>,
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
                        match serde_json::from_str::<UserOpMessage>(payload) {
                            Ok(event) => {
                                println!("📥 Received UserOpMessage for hash: {:?}", event.user_op["userOpHash"]);
                                if let Err(e) = db.upsert_user_op_message(event).await {
                                    eprintln!("❌ Failed to upsert UserOpMessage into Timescale: {:?}", e);
                                }
                            }
                            Err(e) => eprintln!("❌ Failed to deserialize UserOpMessage: {:?}", e),
                        }
                    }
                }
                Err(e) => eprintln!("❌ Kafka error: {:?}", e),
            }
        }
    });
}
