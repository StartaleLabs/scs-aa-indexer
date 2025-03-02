use serde::Deserialize;
use std::{collections::HashMap, fs};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub chains: HashMap<String, ChainConfig>,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ContractConfig {
    pub name: String,
    pub address: String,
    pub events: Vec<EventConfig>,
}

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub indexer_name: String,
}

#[derive(Debug,Clone, Deserialize)]
pub struct ChainConfig {
    pub active: bool,
    pub rpc_url: String,
    pub chain_id: u64,
    pub block_time: u64,
    pub polling_blocks: u64,
    pub contracts: Vec<ContractConfig>,
    pub entrypoints: Vec<EntryPointConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventConfig {
    pub signature: String,    // ✅ Event signature hash (e.g., "0x4962...")
    pub name: String,         // ✅ Event name (e.g., "UserOperationEvent")
    pub params: Vec<String>,  // ✅ Event parameter types (e.g., ["bytes32", "address", ...])
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub use_redis: bool,
    pub redis_url: String,
    pub use_kafka: bool,
    pub kafka_broker: String,
}


#[derive(Debug,Clone, Deserialize)]
pub struct EntryPointConfig {
    pub version: String,
    pub contract_address: String,
    pub events: Vec<EventConfig>,
}

impl Config {
    pub fn load() -> Self {
        let config_contents = fs::read_to_string("config/config.toml")
            .expect("Failed to read config file");

        toml::from_str(&config_contents)
            .expect("Failed to parse config file")
    }
}