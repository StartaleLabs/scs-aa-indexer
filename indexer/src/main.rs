use consumer::kafka_consumer::start_kafka_consumer;
use storage::time_scale::TimescaleStorage;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use sqlx::migrate::Migrator;

use tracing_subscriber;

mod config;
mod listener;
mod processor;
mod storage;
mod consumer;

use crate::listener::listener::EventListener;
use crate::processor::processor::ProcessEvent;
use crate::config::config::Config;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load();
    println!("🔧 Configuration loaded, starting indexer: {:?}", &config.general.indexer_name);
    let (log_sender, log_receiver) = mpsc::channel(100);

    let db = Arc::new(TimescaleStorage::new(&config.storage.timescale_db_url).await);
    MIGRATOR.run(db.get_pg_pool()).await.expect("DB migration failed");

    let kafka_storage: Arc<TimescaleStorage> = Arc::clone(&db);
    let indexer_storage = Arc::clone(&db);

    let kafka_group_id = config.storage.kafka_group_id.clone();
    let kafka_broker = config.storage.kafka_broker.clone();
    let kafka_topics = config.storage.kafka_topics.clone();

    /*
    Start Kafka consumer
    */
    start_kafka_consumer(
        &kafka_broker,
        &kafka_topics[0],
        &kafka_group_id,
        kafka_storage,
    );

    // Start onchain event polling per chain
    for (chain_name, chain) in config.chains.clone() {
        if chain.active {
            let rpc_url = chain.rpc_url.clone();
            let poll_interval = chain.block_time * chain.polling_blocks;
            let log_sender = log_sender.clone();
            let chain_clone = chain.clone();

            tokio::spawn(async move {
                let event_listener = EventListener::new(&rpc_url).await;

                loop {
                    println!("🔍 Listening for events on {}...", chain_name);
                    event_listener.listen_events(&chain_clone, log_sender.clone()).await;
                    sleep(Duration::from_secs(poll_interval)).await;
                }
            });
        }
    }

    // Start log processor
    tokio::spawn(async move {
        let event_processor = ProcessEvent::new(&config, indexer_storage);
        event_processor.process(log_receiver).await;
    });

    // Keep alive
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}