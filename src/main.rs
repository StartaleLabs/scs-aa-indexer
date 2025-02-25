use tokio::sync::mpsc;
use tokio::task;

mod config;
mod listener;
mod processor;

use crate::listener::listener::EventListener;
use crate::processor::processor::ProcessEvent;
use crate::config::config::Config;

#[tokio::main]
async fn main() {
    let config = Config::load();
    println!("Loaded configuration: {:?}", config);

    let rpc_url = &config.chains.minato.rpc_url;
    let contract_address = config.chains.minato.contract_address.clone();

    // 🔹 Create a channel for event logs (Sender & Receiver)
    let (log_sender, log_receiver) = mpsc::channel(100);

    // 🔹 Initialize EventListener
    let event_listener = EventListener::new(&rpc_url).await;

    // 🔹 Initialize Event Processor
    let event_processor = ProcessEvent::new();

    // 🔹 Spawn the event listener in a separate Tokio task
    let listener_handle = task::spawn(async move {
        event_listener.listen(&contract_address, log_sender).await;
    });

    // 🔹 Spawn the event processor in another Tokio task
    let processor_handle = task::spawn(async move {
        event_processor.process(log_receiver).await;
    });

    // 🔹 Wait for both tasks to finish
    let _ = tokio::join!(listener_handle, processor_handle);
}
