
use tokio::sync::mpsc;
use alloy::primitives::B256;

use std::collections::HashMap;
use std::sync::Arc;
use alloy::hex;
use crate::app::AppContext;
use crate::{
    config::config::Config, 
    processor::handler::process_event
};
use crate::{
    storage::Storage,
    cache::Cache,
    model::event::Event,
};

pub struct ProcessEvent<S, C> 
where
    S: Storage + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
    event_map: HashMap<B256, (String, Vec<String>)>,
    app: Arc<AppContext<S, C>>,
}

impl<S, C> ProcessEvent<S, C>
where
    S: Storage + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
    // **Initialize Processor with Dynamic Event Mapping**
    pub fn new(config: &Config, app:Arc<AppContext<S, C>>) -> Self {
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
        }
        Self { event_map, app }
    }

    // **Process Incoming Logs Dynamically**
    pub async fn process(&self, mut receiver: mpsc::Receiver<Event>) {
        let mut previous_event: Option<Event> = None;

        while let Some(event) = receiver.recv().await {
            if let Some(event_signature) = event.log.topics().first() {
                if let Some((event_name, _params)) = self.event_map.get(event_signature) {
                    tracing::info!("✅ Processing Event: {}", event_name);
                    process_event(event_name, &event, &mut previous_event, Arc::clone(&self.app)).await;
                } else {
                    tracing::info!("⚠️ Unknown event signature: {:?}", event_signature);
                }
            } else {
                tracing::info!("⚠️ Log has no topics.");
            }
        }
    }
}