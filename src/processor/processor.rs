use alloy_sol_types::SolEvent;
use tokio::sync::mpsc;
use hex_literal::hex;
use alloy::{
    rpc::types::Log as RpcLog,
    primitives::{
        B256,
        Log as AlloyLog
    }
};
use crate::processor::events::{
    GasBalanceDeducted,
    UserOperationEvent,
    UserOperationSponsored,
    RefundProcessed,
};

pub struct ProcessEvent {}

impl ProcessEvent {
    /// **Create a new ProcessEvent instance**
    pub fn new() -> Self {
        Self {}
    }

    /// **Process incoming logs asynchronously**
    pub async fn process(&self, mut receiver: mpsc::Receiver<RpcLog>) {
        let mut previous_log: Option<RpcLog> = None;

        while let Some(log) = receiver.recv().await {
            if let Some(event_signature) = log.topics().first() {
                match event_signature {
                    sig if *sig == B256::from_slice(&hex!("b51885f42df18ff2d99621fa3752090f501b08a1b746ad11ecc8fa00e068b1db")) => {
                        println!("✅ Processing GasBalanceDeducted...");
                        let alloy_log = AlloyLog::from(log.clone());
                        if let Ok(event) = GasBalanceDeducted::decode_log(&alloy_log, true) {
                            println!(
                                "Decoded GasBalanceDeducted::\n\
                                - user: {:?}\n\
                                - amount: {}\n\
                                - premium: {}\n\
                                - mode: {}",
                                event.user, 
                                event.amount, 
                                event.premium, 
                                event.mode
                            );
                        }
                        previous_log = Some(log.clone());
                    }
                    sig if *sig == B256::from_slice(&hex!("49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f")) => {
                        if let Some(gas_deducted_log) = previous_log.take() {
                            let alloy_log = AlloyLog::from(log.clone());
                            println!("✅ Matched UserOperationEvent for GasBalanceDeducted");
                            if let Ok(event) = UserOperationEvent::decode_log(&alloy_log, true) {
                                println!(
                                    "Decoded UserOperationEvent:\n\
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
                            }
                        } else {
                            println!("⚠️ UserOperationEvent found but no matching GasBalanceDeducted.");
                        }
                    }
                    sig if *sig == B256::from_slice(&hex!("94139248bcc22ab7c689ff34422119f69e04a937052f28621797cb5f69c45af7")) => {
                        println!("✅ Processing UserSponsored event...");
                        let alloy_log: AlloyLog = AlloyLog::from(log.clone());
                        if let Ok(event) = UserOperationSponsored::decode_log(&alloy_log, true) {
                            println!(
                                "UserOperation sponsered from paymaster:\n\
                                - userOpHash: {:?}\n\
                                - address: {}",
                                event.userOpHash, event.address
                            );
                        }
                    }
                    sig if *sig == B256::from_slice(&hex!("3367befd2b2f39615cd79917c2153263c4af1d3945ec003e5d5bfc13a8d85833")) => {
                        println!("✅ Processing GasRefunded event...");
                        let alloy_log: AlloyLog = AlloyLog::from(log.clone());
                        if let Ok(event) = RefundProcessed::decode_log(&alloy_log, true) {
                            println!(
                                "Decoded RefundProcessed:\n\
                                - user: {:?}\n\
                                - amount: {}",
                                event.user, event.amount
                            );
                        }
                    }
                    _ => println!("⚠️ Unknown event type"),
                }
            } else {
                println!("⚠️ Log has no topics.");
            }
        }
    }
}
