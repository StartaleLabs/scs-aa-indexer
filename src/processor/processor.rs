use tokio::sync::mpsc;
use hex_literal::hex;
use alloy::rpc::types::Log;
use alloy::primitives::B256;

/// **ProcessEvent Struct**
pub struct ProcessEvent {}

impl ProcessEvent {
    /// **Create a new ProcessEvent instance**
    pub fn new() -> Self {
        Self {}
    }

    /// **Process incoming logs asynchronously**
    pub async fn process(&self, mut receiver: mpsc::Receiver<Log>) {
        while let Some(log) = receiver.recv().await {
            if let Some(event_signature) = log.topics().first() {
                match event_signature {
                    sig if *sig == B256::from_slice(&hex!("")) => {
                        println!("✅ Processing SponsorDeposited event...");
                        // Add further processing logic here
                    }
                    sig if *sig == B256::from_slice(&hex!("")) => {
                        println!("✅ Processing UserSponsored event...");
                        // Add further processing logic here
                    }
                    sig if *sig == B256::from_slice(&hex!("")) => {
                        println!("✅ Processing GasRefunded event...");
                        // Add further processing logic here
                    }
                    _ => println!("⚠️ Unknown event type"),
                }
            } else {
                println!("⚠️ Log has no topics.");
            }
        }
    }
}
