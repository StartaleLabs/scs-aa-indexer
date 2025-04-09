use alloy_sol_types::SolEvent;
use alloy::{
    primitives::{Address, Log as AlloyLog, B256, U256},
    rpc::types::Log as RpcLog,
};
use scs_aa_indexer::events::events::{
    GasBalanceDeducted, RefundProcessed, UserOperationEvent, UserOperationSponsored, PaidGasInTokens, CombinedUserOpEvent
};
use crate::storage::Storage;

// **Process a log based on the event name**
pub async fn process_event<S: Storage> (event_name: &str, log: &RpcLog, previous_log: &mut Option<RpcLog>, storage: &S) {
    let alloy_log = AlloyLog::from(log.clone());

    match event_name {
        "GasBalanceDeducted" | "PaidGasInTokens" => {
            // Store this log to be paired with the next UserOperationEvent
            *previous_log = Some(log.clone());
        }
        "UserOperationEvent" => {
            if let Some(prev_log) = previous_log.take() {
                let prev_log = AlloyLog::from(prev_log.clone());

                let mut combine_log = CombinedUserOpEvent {
                    user_op_hash: B256::ZERO,
                    sender: Address::ZERO,
                    paymaster: Address::ZERO,
                    paymaster_type: String::new(),
                    nonce: U256::ZERO,
                    success: false,
                    actual_gas_cost: U256::ZERO,
                    actual_gas_used: U256::ZERO,
                    deducted_user: None,
                    deducted_amount: None,
                    deducted_premium: None,
                    token: None,
                    token_charge: None,
                    applied_markup: None,
                    exchange_rate: None,
                };

                // Check if the previous log was a GasBalanceDeducted event from sponsership paymaster
                if let Ok(event) = GasBalanceDeducted::decode_log(&prev_log, true) {
                    combine_log.deducted_user = Some(event.user);
                    combine_log.deducted_amount = Some(event.amount);
                    combine_log.deducted_premium = Some(event.premium);
                    combine_log.paymaster_type = "SPONSORSHIP".to_string();
                }

                // Check if the previous log was a PaidGasInTokens event from token paymaster
                if let Ok(event) = PaidGasInTokens::decode_log(&prev_log, true) {
                    combine_log.deducted_user = Some(event.user);
                    combine_log.token = Some(event.token);
                    combine_log.token_charge = Some(event.tokenCharge);
                    combine_log.applied_markup = Some(event.appliedMarkup);
                    combine_log.exchange_rate = Some(event.exchangeRate);
                    combine_log.paymaster_type = "Token".to_string();
                }

                let user_op_log = AlloyLog::from(log.clone());
                println!("✅ Matched UserOperationEvent for previous event");
                if let Ok(event) = UserOperationEvent::decode_log(&user_op_log, false) {

                    combine_log.user_op_hash = event.userOpHash;
                    combine_log.sender = event.sender;
                    combine_log.paymaster = event.paymaster;
                    combine_log.nonce = event.nonce;
                    combine_log.success = event.success;
                    combine_log.actual_gas_cost = event.actualGasCost;
                    combine_log.actual_gas_used = event.actualGasUsed;
                    
                    println!("✅ Combined Event: {:?}", &combine_log);
                    storage.store(&combine_log, "CombinedUserOperation").await;
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
