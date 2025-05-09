
use alloy::rpc::types::Log;

#[derive(Clone)]
pub struct Event {
    pub chain_id: u32,
    pub log: Log
}
