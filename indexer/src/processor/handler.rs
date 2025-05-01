use std::sync::Arc;

use alloy_sol_types::SolEvent;
use alloy::{
    primitives::Log as AlloyLog,
    rpc::types::Log as RpcLog,
};
use chrono::Utc;
use indexer::events::events::{
    GasBalanceDeducted, RefundProcessed, UserOperationEvent, UserOperationSponsored, PaidGasInTokens
};
use serde_json::json;
use crate::{consumer::kakfa_message::{UserOpMessage, Status}, storage::Storage};

// **Process a log based on the event name**
pub async fn process_event<S: Storage> (event_name: &str, log: &RpcLog, previous_log: &mut Option<RpcLog>, app: Arc<AppContext<S>>,) {
    let alloy_log = AlloyLog::from(log.clone());

    match event_name {
        "GasBalanceDeducted" | "PaidGasInTokens" => {
            // Store this log to be paired with the next UserOperationEvent
            *previous_log = Some(log.clone());
        }
        "UserOperationEvent" => {
            if let Some(prev_log) = previous_log.take() {
                let prev_log = AlloyLog::from(prev_log.clone());

                let mut meta = serde_json::Map::new();
                let mut paymaster_type = "UNKNOWN".to_string();
                let mut token_address = String::new();

                if let Ok(event) = GasBalanceDeducted::decode_log(&prev_log, true) {
                    meta.insert("deductedUser".to_string(), json!(event.user));
                    meta.insert("deductedAmount".to_string(), json!(event.amount));
                    meta.insert("premium".to_string(), json!(event.premium));
                    paymaster_type = "SPONSORSHIP".to_string();
                }

                if let Ok(event) = PaidGasInTokens::decode_log(&prev_log, true) {
                    meta.insert("deductedUser".to_string(), json!(event.user));
                    meta.insert("token".to_string(), json!(event.token));
                    meta.insert("tokenCharge".to_string(), json!(event.tokenCharge));
                    meta.insert("appliedMarkup".to_string(), json!(event.appliedMarkup));
                    meta.insert("exchangeRate".to_string(), json!(event.exchangeRate));
                    paymaster_type = "TOKEN".to_string();
                    token_address = format!("{:?}", event.token);
                }

                let user_op_log = AlloyLog::from(log.clone());
                tracing::info!("✅ Matched UserOperationEvent for previous event");

                if let Ok(event) = UserOperationEvent::decode_log(&user_op_log, false) {
                    meta.insert("actualGasCost".to_string(), json!(event.actualGasCost));
                    meta.insert("actualGasUsed".to_string(), json!(event.actualGasUsed));

                    let msg = UserOpMessage {
                        project_id: None,
                        paymaster_mode: Some(paymaster_type),
                        paymaster_id: None,
                        token_address: Some(token_address),
                        fund_type: None,
                        chain_id: None,
                        policy_id: None,
                        native_usd_price: None,
                        enabled_limits: None,
                        status: if event.success {
                            Status::Success
                        } else {
                            Status::Failed
                        },
                        user_op_hash: format!("{:?}", event.userOpHash),
                        data_source: Some("Indexer".to_string()),
                        timestamp: Utc::now().to_rfc3339(),
                        user_op: json!({
                            "sender": format!("{:?}", event.sender),
                            "paymaster": format!("{:?}", event.paymaster),
                            "nonce": event.nonce.to_string(),
                        }),
                        meta_data: Some(json!(meta)),
                    };
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
