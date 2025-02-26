use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

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
    let pm_contract_address = config.chains.minato.contract_address.clone();
    let entry_point_address = config.entrypoint.contract_address.clone();

    // 🔹 Create a channel for event logs (Sender & Receiver)
    let (log_sender, log_receiver) = mpsc::channel(100);

    // 🔹 Initialize EventListener
    let event_listener = EventListener::new(&rpc_url).await;

    // 🔹 Initialize Event Processor
    let event_processor = ProcessEvent::new();

    tokio::spawn(async move {
        loop {
            println!("🔍 Listening for events...");
            event_listener.listen(vec![pm_contract_address.clone(), entry_point_address.clone()], log_sender.clone()).await;

            sleep(Duration::from_secs(config.general.polling_interval)).await;
        }
    });

    tokio::spawn(async move {
        event_processor.process(log_receiver).await;
    });

    // 🔹 Keep the main thread alive
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
