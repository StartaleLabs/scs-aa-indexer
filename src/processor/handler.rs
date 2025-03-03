use alloy_sol_types::SolEvent;
use alloy::{
    primitives::Log as AlloyLog,
    rpc::types::Log as RpcLog,
};
use scs_aa_indexer::events::events::{
    GasBalanceDeducted, RefundProcessed, UserOperationEvent, UserOperationSponsored,
};
use crate::storage::Storage;

// **Process a log based on the event name**
pub async fn process_event<S: Storage> (event_name: &str, log: &RpcLog, previous_log: &mut Option<RpcLog>, storage: &S) {
    let alloy_log = AlloyLog::from(log.clone());

    match event_name {
        "GasBalanceDeducted" => {
            *previous_log = Some(log.clone());
        }
        "UserOperationEvent" => {
            if let Some(prev_log) = previous_log.take() {
                let gas_balance_deducted_log = AlloyLog::from(prev_log.clone());
                if let Ok(event) = GasBalanceDeducted::decode_log(&gas_balance_deducted_log, true) {
                    println!(
                        "✅ Decoded GasBalanceDeducted:\n\
                        - user: {:?}\n\
                        - amount: {}\n\
                        - premium: {}\n\
                        - mode: {}",
                        event.user, event.amount, event.premium, event.mode
                    );
                    storage.store(&event, "GasBalanceDeducted").await;
                }
                let user_op_log = AlloyLog::from(log.clone());
                println!("✅ Matched UserOperationEvent for previous event");
                if let Ok(event) = UserOperationEvent::decode_log(&user_op_log, false) {
                    println!(
                        "✅ Decoded UserOperationEvent:\n\
                        - userOpHash: {:?}\n\
                        - sender: {:?}\n\
                        - paymaster: {:?}\n\
                        - nonce: {}\n\
                        - success: {}\n\
                        - actualGasCost: {}\n\
                        - actualGasUsed: {}",
                        event.userOpHash,
                        event.sender,
                        event.paymaster,
                        event.nonce,
                        event.success,
                        event.actualGasCost,
                        event.actualGasUsed
                    );
                    storage.store(&event, "UserOperationEvent").await;
                    *previous_log = None;
                }
            } else {
                println!("⚠️ UserOperationEvent found but no previous log matched.");
            }
        }
        "UserOperationSponsored" => {
            if let Ok(event) = UserOperationSponsored::decode_log(&alloy_log, true) {
                println!(
                    "✅ UserOperation Sponsored:\n\
                    - userOpHash: {:?}\n\
                    - user: {:?}",
                    event.userOpHash, event.user
                );
                storage.store(&event, "UserOperationSponsored").await;
            }
        }
        "RefundProcessed" => {
            if let Ok(event) = RefundProcessed::decode_log(&alloy_log, true) {
                println!(
                    "✅ Decoded RefundProcessed:\n\
                    - user: {:?}\n\
                    - amount: {}",
                    event.user, event.amount
                );
                storage.store(&event, "RefundProcessed").await;
            }
        }
        _ => {
            println!("⚠️ Unrecognized event: {:?}", event_name);
        }
    }
}
