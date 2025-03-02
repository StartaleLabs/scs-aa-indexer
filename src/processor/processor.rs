
use tokio::sync::mpsc;
use alloy::{
    rpc::types::Log as RpcLog,
    primitives::{
        B256,
        Log as AlloyLog
    }
};


use std::collections::HashMap;
use alloy::hex;
use crate::{
    config::config::Config, 
    processor::handler::process_event
};

pub struct ProcessEvent {
    event_map: HashMap<B256, (String, Vec<String>)>, // Maps event signature → (event name, param types)
}

impl ProcessEvent {
    /// **Initialize Processor with Dynamic Event Mapping**
    pub fn new(config: &Config) -> Self {
        let mut event_map = HashMap::new();

        // 🔹 Iterate over all chains & their contracts
        for (_, chain) in &config.chains {
            for contract in &chain.contracts {
                for event in &contract.events {
                    let event_sig = B256::from_slice(
                        &hex::decode(event.signature.trim_start_matches("0x")).expect("Invalid event signature"),
                    );
                    event_map.insert(event_sig, (event.name.clone(), event.params.clone()));
                }
            }

            // 🔹 Iterate over entrypoints
            for entrypoint in &chain.entrypoints {
                for event in &entrypoint.events {
                    let event_sig = B256::from_slice(
                        &hex::decode(event.signature.trim_start_matches("0x")).expect("Invalid event signature"),
                    );
                    event_map.insert(event_sig, (event.name.clone(), event.params.clone()));
                }
            }
        }

        Self { event_map }
    }

    /// **Process Incoming Logs Dynamically**
    pub async fn process(&self, mut receiver: mpsc::Receiver<RpcLog>) {
        let mut previous_log: Option<RpcLog> = None;

        while let Some(log) = receiver.recv().await {
            if let Some(event_signature) = log.topics().first() {
                if let Some((event_name, _params)) = self.event_map.get(event_signature) {
                    println!("✅ Processing Event: {}", event_name);
                    process_event(event_name, &log, &mut previous_log);
                } else {
                    println!("⚠️ Unknown event signature: {:?}", event_signature);
                }
            } else {
                println!("⚠️ Log has no topics.");
            }
        }
    }
}