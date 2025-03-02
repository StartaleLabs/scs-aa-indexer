use alloy_sol_types::SolEvent;
use alloy::{
    primitives::Log as AlloyLog,
    rpc::types::Log as RpcLog,
};
use scs_aa_indexer::events::events::{
    GasBalanceDeducted, RefundProcessed, UserOperationEvent, UserOperationSponsored,
};

/// **Process a log based on the event name**
pub fn process_event(event_name: &str, log: &RpcLog, previous_log: &mut Option<RpcLog>) {
    let alloy_log = AlloyLog::from(log.clone());

    match event_name {
        "GasBalanceDeducted" => {
            if let Ok(event) = GasBalanceDeducted::decode_log(&alloy_log, true) {
                println!(
                    "✅ Decoded GasBalanceDeducted:\n\
                    - user: {:?}\n\
                    - amount: {}\n\
                    - premium: {}\n\
                    - mode: {}",
                    event.user, event.amount, event.premium, event.mode
                );
            }
            *previous_log = Some(log.clone());
        }
        "UserOperationEvent" => {
            if let Some(_) = previous_log.take() {
                let prev_alloy_log = AlloyLog::from(log.clone());
                println!("✅ Matched UserOperationEvent for previous event");
                if let Ok(event) = UserOperationEvent::decode_log(&prev_alloy_log, false) {
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
            }
        }
        _ => {
            println!("⚠️ Unrecognized event: {:?}", event_name);
        }
    }
}
