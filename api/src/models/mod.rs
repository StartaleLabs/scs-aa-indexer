// api/models/mod.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UserOperationRecord {
    pub user_op_hash: String,
    pub user_operation: Option<serde_json::Value>,
    pub paymaster_id: Option<String>,
    pub project_id: Option<String>,
    pub paymaster_mode: Option<String>,
    pub data_source: Option<String>,
    pub status: String,
    pub token_address: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
