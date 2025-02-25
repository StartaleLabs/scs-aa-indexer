use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub chains: ChainsConfig,
    pub storage: StorageConfig,
}


#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub indexer_name: String,
    pub polling_interval: u64,
}

#[derive(Debug, Deserialize)]
pub struct ChainsConfig {
    pub soneium: ChainConfig,
    pub minato: ChainConfig,
}

#[derive(Debug, Deserialize)]
pub struct ChainConfig {
    pub rpc_url: String,
    pub contract_address: String,
    pub chainId : u32,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub use_postgres: bool,
    pub postgres_url: String,
    pub use_redis: bool,
    pub redis_url: String,
    pub use_opensearch: bool,
    pub opensearch_url: String,
    pub use_kafka: bool,
    pub kafka_broker: String,
}

impl Config {
    pub fn load() -> Self {
        let config_contents = fs::read_to_string("config/config.toml")
            .expect("Failed to read config file");

        toml::from_str(&config_contents)
            .expect("Failed to parse config file")
    }
}