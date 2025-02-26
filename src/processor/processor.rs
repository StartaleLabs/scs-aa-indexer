use tokio::sync::mpsc;
use hex_literal::hex;
use alloy::rpc::types::Log;
use alloy::primitives::B256;

pub struct ProcessEvent {}

impl ProcessEvent {
    /// **Create a new ProcessEvent instance**
    pub fn new() -> Self {
        Self {}
    }

    /// **Process incoming logs asynchronously**
    pub async fn process(&self, mut receiver: mpsc::Receiver<Log>) {
        let mut previous_log: Option<Log> = None;

        while let Some(log) = receiver.recv().await {
            if let Some(event_signature) = log.topics().first() {
                match event_signature {
                    sig if *sig == B256::from_slice(&hex!("b51885f42df18ff2d99621fa3752090f501b08a1b746ad11ecc8fa00e068b1db")) => {
                        println!("✅ Processing GasBalanceDeducted...");
                        previous_log = Some(log.clone());
                    }
                    sig if *sig == B256::from_slice(&hex!("49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f")) => {
                        if let Some(gas_deducted_log) = previous_log.take() {
                            println!(
                                "✅ Matched UserOperationEvent for GasBalanceDeducted {:?} -> {:?}",
                                gas_deducted_log, log
                            );
                        } else {
                            println!("⚠️ UserOperationEvent found but no matching GasBalanceDeducted.");
                        }
                    }
                    sig if *sig == B256::from_slice(&hex!("94139248bcc22ab7c689ff34422119f69e04a937052f28621797cb5f69c45af7")) => {
                        println!("✅ Processing UserSponsored event...");
                        println!("User Sponsored: {:?}", log);
                    }
                    sig if *sig == B256::from_slice(&hex!("3367befd2b2f39615cd79917c2153263c4af1d3945ec003e5d5bfc13a8d85833")) => {
                        println!("✅ Processing GasRefunded event...");
                        println!("Gas Refunded: {:?}", log);
                    }
                    _ => println!("⚠️ Unknown event type"),
                }
            } else {
                println!("⚠️ Log has no topics.");
            }
        }
    }
}
