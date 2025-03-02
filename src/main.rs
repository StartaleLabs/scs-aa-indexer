use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

mod config;
mod listener;
mod processor;
mod storage;

use crate::listener::listener::EventListener;
use crate::processor::processor::ProcessEvent;
use crate::config::config::Config;
use crate::storage::kafka::KafkaStorage;
#[tokio::main]
async fn main() {
    let config = Config::load();
    println!("Loaded configuration: {:?}", &config);

    let (log_sender, log_receiver) = mpsc::channel(100);

    let kafka = KafkaStorage::new(&config.storage.kafka_broker, &config.storage.kafka_topics[0]);

    let event_processor = ProcessEvent::new(&config, kafka);
    let chains = config.chains.clone();
    for (chain_name, chain) in chains {
        if chain.active {
            let rpc_url = &chain.rpc_url;
            let poll_interval = chain.block_time * chain.polling_blocks;
            let event_listener = EventListener::new(rpc_url).await;

            tokio::spawn({
                let log_sender = log_sender.clone();
                let chain = chain.clone();

                async move {
                    loop {
                        println!("🔍 Listening for events on {}...", chain_name);
                        event_listener.listen_events(&chain, log_sender.clone()).await;
                        sleep(Duration::from_secs(poll_interval)).await;
                    }
                }
            });
        }
    }

    tokio::spawn(async move {
        event_processor.process(log_receiver).await;
    });

    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
