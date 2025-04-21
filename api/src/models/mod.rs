// api/models/mod.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UserOperationRecord {
    pub user_op_hash: String,
    pub user_operation: Option<serde_json::Value>,
    pub policyid: Option<String>,
    pub projectid: Option<String>,
    pub paymastermode: Option<String>,
    pub datasource: Option<String>,
    pub status: String,
    pub tokenaddress: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}