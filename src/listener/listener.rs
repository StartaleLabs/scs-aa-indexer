use alloy::{
    network::Ethereum,
    primitives::{Address, B256},
    providers::{Provider, RootProvider},
    rpc::{client::RpcClient, types::{Filter, Log}},
    transports::http::Http,
};
use std::str::FromStr;
use tokio::sync::mpsc;
use url::Url;
use crate::config::config::ChainConfig;

/// **EventListener Struct**
pub struct EventListener {
    provider: RootProvider<Ethereum>
}

impl EventListener {
    /// **Initialize the EventListener**
    pub async fn new(rpc_url: &str) -> Self {
        let url = Url::parse(rpc_url).expect("Invalid RPC URL");

        // **Initialize HTTP Transport**
        let transport = Http::new(url);

        // **Create RPC Client**
        let rpc_client = RpcClient::new(transport, true);

        // **Create RootProvider**
        let provider = RootProvider::new(rpc_client);

        Self { provider }
    }

    pub async fn listen_events(&self, chain_config: &ChainConfig, sender: mpsc::Sender<Log>) {
       if !chain_config.active {
           return;
       }

       let contract_addresses: Vec<Address> = chain_config
            .contracts
            .iter()
            .map(|c| Address::from_str(&c.address).expect("Invalid contract address"))
            .collect();

       
        let entrypoint_addresses: Vec<Address> = chain_config
            .entrypoints
            .iter()
            .map(|ep| Address::from_str(&ep.contract_address).expect("Invalid entrypoint address"))
            .collect();

        // **Combine Paymaster & EntryPoint Addresses**
        let all_contract_addresses = [contract_addresses, entrypoint_addresses].concat();

        let mut event_signatures: Vec<B256> = Vec::new();

        // **Extract Events from Contracts**
        for contract in &chain_config.contracts {
            println!("Listening to contract: {:?}", contract.name);
            for event in &contract.events {
                event_signatures.push(B256::from_str(&event.signature).expect("Invalid event signature"));
            }
        }

        // **Extract Events from EntryPoints**
        for entrypoint in &chain_config.entrypoints {
            println!("Listening to contract: EntryPoint Version {}", entrypoint.version);
            for event in &entrypoint.events {
                event_signatures.push(B256::from_str(&event.signature).expect("Invalid event signature"));
            }
        }

        let latest_block = self.provider.get_block_number().await.unwrap();
        let from_block = latest_block - chain_config.polling_blocks;
        let filter = Filter::new()
            .address(all_contract_addresses)
            .event_signature(event_signatures)
            .from_block(from_block)
            .to_block(latest_block);

        // **Retrieve Logs**
        match self.provider.get_logs(&filter).await {
            Ok(logs) => {
                for log in logs {
                    eprint!("Log: {:?}", log);
                    if sender.send(log).await.is_err() {
                        eprintln!("Failed to send log to channel");
                    }
                }
            }
            Err(e) => eprintln!("Error fetching logs: {:?}", e),
        }
    }
    
}
