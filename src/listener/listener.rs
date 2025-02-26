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
    pub async fn listen(&self, contracts: Vec<String>, sender: mpsc::Sender<Log>) {

        let contract_addresses = contracts.iter().map(|addr| Address::from_str(addr).expect("Invalid contract address")).collect::<Vec<Address>>();

        // **Event Topics** (Replace with actual event signatures)

        // Event for User operation from Entry point
        let user_operation_event = B256::from_str("0x49628fd1471006c1482da88028e9ce4dbb080b815c9b0344d39e5a8e6ec1419f").unwrap();
        
        // Events for User Sponsored and Gas Refunded from Paymaster
        let gas_balance_deducted = B256::from_str("0xb51885f42df18ff2d99621fa3752090f501b08a1b746ad11ecc8fa00e068b1db").unwrap();
        let user_sponsored= B256::from_str("0x94139248bcc22ab7c689ff34422119f69e04a937052f28621797cb5f69c45af7").unwrap();
        let refund_processed = B256::from_str("0x3367befd2b2f39615cd79917c2153263c4af1d3945ec003e5d5bfc13a8d85833").unwrap();

        // Filter to fetch paymaster events
        let latest_block = self.provider.get_block_number().await.unwrap();
        let filter = Filter::new()
            .address(contract_addresses)
            .event_signature(vec![gas_balance_deducted, user_operation_event, user_sponsored, refund_processed])
            .from_block((latest_block - 50)) // Fetch last 100 blocks
            .to_block(latest_block);
        
        // **Retrieve Logs**
        match self.provider.get_logs(&filter).await {
            Ok(logs) => {
                for log in logs {
                    eprint!("Log: {:?}", log);
                    // **Send Log to Channel**
                    if sender.send(log).await.is_err() {
                        eprintln!("Failed to send log to channel");
                    }
                }
            }
            Err(e) => eprintln!("Error fetching logs: {:?}", e),
        }
    }
}
