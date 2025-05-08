use serde::Deserialize;
use std::{collections::HashMap, fs, env};
use dotenv::dotenv;

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
    pub chain_id: u32,
    pub block_time: u64,
    pub polling_blocks: u64,
    pub contracts: Vec<ContractConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EventConfig {
    pub signature: String,    // ✅ Event signature hash (e.g., "0x4962...")
    pub name: String,         // ✅ Event name (e.g., "UserOperationEvent")
    pub params: Vec<String>,  // ✅ Event parameter types (e.g., ["bytes32", "address", ...])
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub kafka_broker: String,
    pub kafka_topics: Vec<String>,
    pub kafka_group_id: String,
    pub timescale_db_url: String,
    pub redis_url: String,
}

impl Config {
    pub fn load() -> Self {
        dotenv().ok();
        let mut config = String::new();
        if let Ok(config_path) = env::var("CONFIG_PATH") {
            config = config_path;
        }
        // 🔹 **Load Configuration from config.toml file**
        let config_contents = fs::read_to_string(config)
            .expect("Failed to read config file");

        let mut config: Config =
            toml::from_str(&config_contents).expect("Failed to parse config file");

        // 🔹 **Override Chain RPC URLs Dynamically from ENV**
        for (chain_name, chain_config) in config.chains.iter_mut() {
            let env_var_name: String = format!("{}_RPC_URL", chain_name.to_uppercase());
            if let Ok(rpc_url) = env::var(&env_var_name) {
                chain_config.rpc_url = rpc_url;
            }
        }

        // 🔹 **Override Storage Configuration Dynamically ENV**
        if let Ok(timescale_db_url) = env::var("TIMESCALE_DB_URL") {
            config.storage.timescale_db_url = timescale_db_url;
        }

        if let Ok(kafka_broker) = env::var("KAFKA_BROKER") {
            config.storage.kafka_broker = kafka_broker;
        }
        if let Ok(kafka_topics) = env::var("KAFKA_TOPICS") {
            config.storage.kafka_topics = kafka_topics.split(',').map(String::from).collect();
        }
        if let Ok(kafka_group_id) = env::var("KAFKA_GROUP_ID") {
            config.storage.kafka_group_id = kafka_group_id;
        }
        if let Ok(redis_url) = env::var("REDIS_URL") {
            config.storage.redis_url = redis_url;
        }
        
        config
    }
}