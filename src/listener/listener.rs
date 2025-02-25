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

    /// **Listen for Events on Paymaster Contract**
    pub async fn listen(&self, contract_address: &str, sender: mpsc::Sender<Log>) {
        let contract_addr = Address::from_str(contract_address).expect("Invalid contract address");

        // **Event Topics** (Replace with actual event signatures)
        let sponsor_deposited = B256::from_str("0xYourEventSignatureForSponsorDeposited").unwrap();
        let user_sponsored = B256::from_str("0xYourEventSignatureForUserSponsored").unwrap();
        let gas_refunded = B256::from_str("0xYourEventSignatureForGasRefunded").unwrap();

        let filter = Filter::new()
            .address(vec![contract_addr])
            .topic1(sponsor_deposited)
            .topic2(user_sponsored)
            .topic3(gas_refunded);

        // **Retrieve Logs**
        match self.provider.get_logs(&filter).await {
            Ok(logs) => {
                for log in logs {
                    if sender.send(log).await.is_err() {
                        eprintln!("Failed to send log to channel");
                    }
                }
            }
            Err(e) => eprintln!("Error fetching logs: {:?}", e),
        }
    }
}
