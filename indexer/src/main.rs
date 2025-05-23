mod app;
mod cache;
mod config;
mod consumer;
mod listener;
mod model;
mod processor;
mod storage;
mod utils;
use futures_util::FutureExt;
use sqlx::migrate::Migrator;
use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing_subscriber;

use app::AppContext;
use cache::redis::RedisCoordinator;
use config::config::Config;
use consumer::kafka_consumer::start_kafka_consumer;
use listener::listener::EventListener;
use processor::processor::ProcessEvent;
use storage::time_scale::TimescaleStorage;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

fn spawn_safe<F>(fut: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    tokio::spawn(async move {
        if let Err(err) = AssertUnwindSafe(fut).catch_unwind().await {
            tracing::error!("🚨 A task panicked: {:?}", err);
        }
    });
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::load();
    tracing::info!(
        "🔧 Configuration loaded, starting indexer: {:?}",
        &config.general.indexer_name
    );

    let (log_sender, log_receiver) = mpsc::channel(100);

    // ✅ Initialize DB and Redis
    let db = Arc::new(TimescaleStorage::new(&config.storage.timescale_db_url).await);
    let redis = Arc::new(RedisCoordinator::new(&config.storage.redis_url));

    // ✅ DB migration
    MIGRATOR.run(db.get_pg_pool()).await.unwrap_or_else(|e| {
        tracing::error!("❌ DB migration failed: {:?}", e);
        std::process::exit(1); // Fail fast
    });

    // ✅ Wrap both into shared AppContext
    let app: Arc<_> = Arc::new(AppContext::new(db, redis));
    let indexer_app = Arc::clone(&app);

    // ✅ Start Kafka consumer
    let kafka_broker = config.storage.kafka_broker.clone();
    let kafka_topics = config.storage.kafka_topics.clone();
    let kafka_group_id = config.storage.kafka_group_id.clone();

    tracing::info!("🟢 Starting Kafka consumer...");
    spawn_safe(async move {
        loop {
            let kafka_app = Arc::clone(&app);
            let result = AssertUnwindSafe(
                start_kafka_consumer(&kafka_broker, &kafka_topics[0], &kafka_group_id, kafka_app)
            )
            .catch_unwind()
            .await;
    
            if let Err(err) = result {
                tracing::error!("🔥 Kafka consumer panicked, restarting... {:?}", err);
            } else {
                tracing::warn!("⚠️ Kafka consumer exited unexpectedly, restarting...");
            }
    
            sleep(Duration::from_secs(5)).await;
        }
    });
    

    // ✅ Spawn per-chain listeners
    for (chain_name, chain) in config.chains.clone() {
        if chain.active {
            let rpc_url = chain.rpc_url.clone();
            let poll_interval = chain.block_time * chain.polling_blocks;
            let log_sender = log_sender.clone();
            let chain_clone = chain.clone();
            let chain_name_clone = chain_name.clone();
            let app_for_chain = Arc::clone(&indexer_app);

            spawn_safe(async move {
                // this loop ensures the listener restarts if it panics
                loop {
                    let result = AssertUnwindSafe(async {
                        let event_listener: EventListener<TimescaleStorage, RedisCoordinator> =
                            EventListener::new(&rpc_url, Arc::clone(&app_for_chain)).await;
                        loop {
                            tracing::info!("🔍 Listening for events on {}...", chain_name_clone);
                            event_listener
                                .listen_events(&chain_clone, log_sender.clone())
                                .await;
                            sleep(Duration::from_secs(poll_interval)).await;
                        }
                    })
                    .catch_unwind()
                    .await;

                    if let Err(err) = result {
                        tracing::error!(
                            "🔥 Chain listener for {} panicked, restarting... {:?}",
                            chain_name_clone, err
                        );
                        sleep(Duration::from_secs(5)).await;
                    }
                }
            });
        }
    }

    // ✅ Log processing
    spawn_safe(async move {
        let event_processor = ProcessEvent::new(&config, indexer_app);
        event_processor.process(log_receiver).await;
    });

    // ✅ Main keep-alive loop
    loop {
        sleep(Duration::from_secs(3600)).await;
    }
}
