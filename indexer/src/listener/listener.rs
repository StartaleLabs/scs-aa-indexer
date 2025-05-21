use alloy::{ network::Ethereum, primitives::{Address, B256}, providers::{Provider, RootProvider}, rpc::{client::RpcClient, types::Filter}, transports::http::Http
};
use std::{str::FromStr, sync::Arc};
use tokio::sync::mpsc;
use url::Url;
use crate::{
    app::AppContext, cache::Cache, config::config::ChainConfig, model::event::Event, storage::Storage
};
use alloy::rpc::types::{BlockId, BlockNumberOrTag, BlockTransactionsKind};


/// **EventListener Struct**
pub struct EventListener<S, C>
where
    S: Storage + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
    provider: RootProvider<Ethereum>,
    app: Arc<AppContext<S, C>>,
}

impl<S, C> EventListener<S, C>
where
    S: Storage + Send + Sync + 'static,
    C: Cache + Send + Sync + 'static,
{
    /// **Initialize the EventListener**
    pub async fn new(rpc_url: &str, app: Arc<AppContext<S, C>>) -> Self {
        let url = Url::parse(rpc_url).expect("Invalid RPC URL");

        // **Initialize HTTP Transport**
        let transport = Http::new(url);

        // **Create RPC Client**
        let rpc_client = RpcClient::new(transport, true);

        // **Create RootProvider**
        let provider = RootProvider::new(rpc_client);

        Self { provider, app }
    }

    pub async fn listen_events(&self, chain_config: &ChainConfig, sender: mpsc::Sender<Event>) {
        if !chain_config.active {
            return;
        }
    
        // -- Contract addresses
        let contract_addresses: Vec<Address> = chain_config
            .contracts
            .iter()
            .map(|c| Address::from_str(&c.address).expect("Invalid contract address"))
            .collect();
    
        // -- Event signatures
        let mut event_signatures: Vec<B256> = Vec::new();
        for contract in &chain_config.contracts {
            tracing::info!("-- Listening to Contract: {} on chainId: {}", contract.name, chain_config.chain_id);
            for event in &contract.events {
                event_signatures.push(B256::from_str(&event.signature).expect("Invalid event signature"));
            }
        }
    
        // -- Get latest and finalized blocks
        let latest_block_number = self.provider.get_block_number().await.unwrap();
        let finalized_block = if chain_config.use_finalized {
            self.provider
                .get_block(
                    BlockId::Number(BlockNumberOrTag::Finalized),
                    BlockTransactionsKind::Hashes,
                )
                .await
                .ok()
                .flatten()
                .map(|b| b.header.inner.number)
        } else {
            None
        };
    
        // -- Compute to_block
        let reorg_buffer = chain_config.reorg_buffer;
        let to_block = finalized_block.unwrap_or_else(|| latest_block_number.saturating_sub(reorg_buffer));
    
        tracing::info!(
            "📍 latest_block: {}, finalized_block: {:?}, reorg_buffer: {}, using_finalized: {}",
            latest_block_number,
            finalized_block,
            reorg_buffer,
            chain_config.use_finalized
        );
    
        // -- Determine from_block
        let from_block = match self.app.cache.get_last_synced_block(chain_config.chain_id).await {
            Ok(Some(last_synced)) => {
                let next_block = last_synced + 1;
                if next_block > to_block {
                    tracing::info!(
                        "⏳ No new blocks to index for chain {} (from {} > to {})",
                        chain_config.chain_id,
                        next_block,
                        to_block
                    );
                    return;
                }
                next_block
            }
            Ok(None) => to_block.saturating_sub(chain_config.polling_blocks),
            Err(e) => {
                tracing::error!("Failed to get last synced block: {:?}", e);
                to_block.saturating_sub(chain_config.polling_blocks)
            }
        };
    
        // Final catch-up check
        if from_block > to_block {
            tracing::info!(
                "⏳ No new blocks to index for chain {} (from {} > to {})",
                chain_config.chain_id,
                from_block,
                to_block
            );
            return;
        }
    
        tracing::info!(
            "📦 Fetching logs for chain {} from block {} to {} (contracts: {}, events: {})",
            chain_config.chain_id,
            from_block,
            to_block,
            contract_addresses.len(),
            event_signatures.len()
        );
    
        // -- Build log filter
        let filter = Filter::new()
            .address(contract_addresses)
            .event_signature(event_signatures)
            .from_block(from_block)
            .to_block(to_block);
    
        // -- Retrieve logs and process
        match self.provider.get_logs(&filter).await {
            Ok(logs) => {
                let mut max_block_seen: Option<u64> = None;
    
                if logs.is_empty() {
                    tracing::warn!(
                        "⚠️ No logs returned for chain {} from {} to {} — advancing anyway",
                        chain_config.chain_id,
                        from_block,
                        to_block
                    );
                    max_block_seen = Some(to_block);
                }
    
                for log in logs {
                    tracing::debug!("ChainID: {}, Log: {:?}", chain_config.chain_id, log);
    
                    if let Some(block_number) = log.block_number {
                        max_block_seen = Some(max_block_seen.map_or(block_number, |max| max.max(block_number)));
                    } else {
                        tracing::warn!("⚠️ Log missing block_number — cannot use for sync progress");
                    }
    
                    if sender
                        .send(Event {
                            chain_id: chain_config.chain_id,
                            log,
                        })
                        .await
                        .is_err()
                    {
                        tracing::error!("❌ Failed to send log to processor. Aborting block update.");
                        return;
                    }
                }
    
                if let Some(max_block) = max_block_seen {
                    if let Err(e) = self
                        .app
                        .cache
                        .set_last_synced_block(chain_config.chain_id, max_block)
                        .await
                    {
                        tracing::error!("⚠️ Failed to set last synced block to {}: {:?}", max_block, e);
                    } else {
                        tracing::info!(
                            "✅ Updated last synced block for chain {} to {}",
                            chain_config.chain_id,
                            max_block
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error fetching logs: {:?}", e);
            }
        }
    }    

}
