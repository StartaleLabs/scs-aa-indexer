use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct UserOpMessage {
    pub project_id: Option<String>,
    pub paymaster_mode: String,
    pub policy_id: Option<String>,
    pub token_address: Option<String>,
    pub status: String,
    pub data_source: String,
    pub timestamp: String,
    pub user_op: serde_json::Value,
    pub meta_data: Option<serde_json::Value>,
    pub user_op_hash: String,
}
