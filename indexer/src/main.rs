mod config;
mod listener;
mod processor;
mod storage;
mod consumer;
mod app;
mod cache;

use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing_subscriber;
use sqlx::migrate::Migrator;

use consumer::kafka_consumer::start_kafka_consumer;
use listener::listener::EventListener;
use processor::processor::ProcessEvent;
use config::config::Config;
use app::AppContext;
use storage::time_scale::TimescaleStorage;
use cache::redis::RedisCoordinator;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load();
    println!("🔧 Configuration loaded, starting indexer: {:?}", &config.general.indexer_name);

    let (log_sender, log_receiver) = mpsc::channel(100);

    // ✅ Initialize DB and Redis
    let db = Arc::new(TimescaleStorage::new(&config.storage.timescale_db_url).await);
    let redis = Arc::new(RedisCoordinator::new(&config.storage.redis_url));

    // ✅ DB migration
    MIGRATOR.run(db.get_pg_pool())
        .await
        .expect("DB migration failed");

    // ✅ Wrap both into shared AppContext
    let app: Arc<_> = Arc::new(AppContext::new(db, redis));
    let kafka_app = Arc::clone(&app);
    let indexer_app = Arc::clone(&app);

    // ✅ Start Kafka consumer
    start_kafka_consumer(
        &config.storage.kafka_broker,
        &config.storage.kafka_topics[0],
        &config.storage.kafka_group_id,
        kafka_app,
    );

    // ✅ Spawn per-chain listeners
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

    // ✅ Process logs
    tokio::spawn(async move {
        let event_processor = ProcessEvent::new(&config, indexer_app);
        event_processor.process(log_receiver).await;
    });

    // ✅ Keep alive
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
