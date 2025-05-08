use std::sync::Arc;

use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::ClientConfig;
use rdkafka::message::Message;
use crate::model::paymaster_type::PaymasterMode;
use crate::{
    model::{user_op_policy::UserOpPolicyData, user_op::UserOpMessage},
    storage::Storage,
    cache::Cache,
};
use super::super::app::AppContext;
pub fn start_kafka_consumer<S, C>(
    brokers: &str,
    topic: &str,
    group_id: &str,
    app: Arc<AppContext<S, C>>,
) 
where
    S: Storage + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
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
                                // ✅ 1. Update Redis
                                if matches!(event.paymaster_mode, Some(PaymasterMode::SPONSORSHIP)) {
                                    if let Some(policy_id) = event.policy_id.clone() {
                                        tracing::info!("🟢 Updating Redis with policy_id: {}", policy_id);
                                        let redis_payload = UserOpPolicyData {
                                            policy_id: Some(policy_id),
                                            native_usd_price: event.native_usd_price.clone(),
                                            sender: event.user_op.get("sender").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                            enabled_limits: event.enabled_limits.clone(),
                                            actual_gas_used: None,
                                            actual_gas_cost: None,
                                        };

                                        if let Err(e) = app.cache.update_userop_policy(&event.user_op_hash, redis_payload).await {
                                            tracing::error!("❌ Failed to update Redis policy: {:?}", e);
                                        }
                                    }
                                }
                                // ✅ 2. Update DB
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
