use std::sync::Arc;

use alloy_sol_types::SolEvent;
use alloy::primitives::Log as AlloyLog;
use chrono::Utc;
use indexer::events::events::{
    GasBalanceDeducted, RefundProcessed, UserOperationEvent, UserOperationSponsored, PaidGasInTokens
};
use serde_json::json;
use crate::{
    app::AppContext, cache::Cache, model::user_op::{Status, UserOpMessage}, model::{paymaster_type::PaymasterMode, user_op_policy::UserOpPolicyData}, storage::Storage,
    model::event::Event,
};

// **Process a log based on the event name**
pub async fn process_event<S, C>(
    event_name: &str,
    event: &Event,
    previous_event: &mut Option<Event>,
    app: Arc<AppContext<S, C>>,
)
where
    S: Storage + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{   
    let alloy_log = AlloyLog::from(event.log.clone());
    match event_name {
        "GasBalanceDeducted" | "PaidGasInTokens" | "UserOperationSponsoredForPostpaid" => {
            // Store this log to be paired with the next UserOperationEvent
            *previous_event = Some(event.clone());
        }
        "UserOperationEvent" => {
            if let Some(prev_event) = previous_event.take() {
                let prev_log = AlloyLog::from(prev_event.log);

                let mut meta = serde_json::Map::new();
                let mut paymaster_type = PaymasterMode::UNKNOWN;
                let mut token_address = String::new();

                if let Ok(log) = GasBalanceDeducted::decode_log(&prev_log, true) {
                    meta.insert("deductedUser".to_string(), json!(log.user.to_string()));
                    meta.insert("deductedAmount".to_string(), json!(log.amount.to_string()));
                    meta.insert("premium".to_string(), json!(log.premium.to_string()));
                    paymaster_type = PaymasterMode::SPONSORSHIP;
                }

                if let Ok(log) = PaidGasInTokens::decode_log(&prev_log, true) {
                    meta.insert("deductedUser".to_string(), json!(log.user.to_string()));
                    meta.insert("token".to_string(), json!(log.token));
                    meta.insert("tokenCharge".to_string(), json!(log.tokenCharge.to_string()));
                    meta.insert("appliedMarkup".to_string(), json!(log.appliedMarkup.to_string()));
                    meta.insert("exchangeRate".to_string(), json!(log.exchangeRate.to_string()));
                    paymaster_type = PaymasterMode::TOKEN;
                    token_address = format!("{:?}", log.token);
                }

                let user_op_log = AlloyLog::from(event.log.clone());
                tracing::info!("✅ Matched UserOperationEvent for previous event");

                if let Ok(log) = UserOperationEvent::decode_log(&user_op_log, false) {
                    meta.insert("actualGasCost".to_string(), json!(log.actualGasCost.to_string()));
                    meta.insert("actualGasUsed".to_string(), json!(log.actualGasUsed.to_string()));

                    let msg = UserOpMessage {
                        org_id: None,
                        credential_id: None,
                        paymaster_mode: Some(paymaster_type.clone()),
                        paymaster_id: None,
                        token_address: Some(token_address),
                        fund_type: None,
                        chain_id: event.chain_id,
                        policy_id: None,
                        native_usd_price: None,
                        enabled_limits: None,
                        status: if log.success {
                            Status::Success
                        } else {
                            Status::Failed
                        },
                        user_op_hash: format!("{:?}", log.userOpHash),
                        data_source: Some("Indexer".to_string()),
                        timestamp: Utc::now().to_rfc3339(),
                        user_op: json!({
                            "sender": format!("{:?}", log.sender),
                            "paymaster": format!("{:?}", log.paymaster),
                            "nonce": log.nonce.to_string(),
                        }),
                        meta_data: Some(json!(meta)),
                    };
                    // ✅ Update Redis with info
                    if paymaster_type == PaymasterMode::SPONSORSHIP {
                        let redis_payload = UserOpPolicyData {
                            policy_id: None,
                            native_usd_price: None,
                            actual_gas_cost: Some(log.actualGasCost.to_string()),
                            actual_gas_used: Some(log.actualGasUsed.to_string()),
                            sender: None,
                            enabled_limits: None, 
                        };
                        println!("redis onchain payload {}", serde_json::to_string(&redis_payload).unwrap());
                        if let Err(e) = app.cache.update_userop_policy(&msg.user_op_hash, redis_payload).await {
                            tracing::error!("❌ Failed to update Redis with indexer data: {:?}", e);
                        }
                    }
                    // ✅ Update Timescale with info
                    println!("userOpMessage {}", serde_json::to_string(&msg).unwrap());
                    app.storage.upsert_user_op_message(msg).await.unwrap_or_else(|e| {
                        tracing::error!("❌ Failed to upsert UserOpMessage into Storage: {:?}", e);
                    });
                }
            } else {
                tracing::warn!("⚠️ UserOperationEvent found but no previous log matched.");
            }
        }
        "UserOperationSponsored" => {
            if let Ok(event) = UserOperationSponsored::decode_log(&alloy_log, true) {
                tracing::info!(
                    "✅ UserOperation Sponsored:\n\
                    - userOpHash: {:?}\n\
                    - user: {:?}",
                    event.userOpHash, event.user
                );
            }
        }
        "RefundProcessed" => {
            if let Ok(event) = RefundProcessed::decode_log(&alloy_log, true) {
                tracing::info!(
                    "✅ Decoded RefundProcessed:\n\
                    - user: {:?}\n\
                    - amount: {}",
                    event.user, event.amount
                );
            }
        }
        _ => {
            tracing::warn!("⚠️ Unrecognized event: {:?}", event_name);
        }
    }
}
