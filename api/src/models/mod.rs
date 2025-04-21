// api/models/mod.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UserOperationRecord {
    pub id: i32,
    pub user_op_hash: String,
    pub user_operation: Option<serde_json::Value>,

    #[sqlx(rename = "policyId")]
    pub policy_id: Option<String>,

    #[sqlx(rename = "projectId")]
    pub project_id: Option<String>,

    #[sqlx(rename = "paymasterMode")]
    pub paymaster_mode: Option<String>,

    #[sqlx(rename = "dataSource")]
    pub data_source: Option<String>,

    pub status: String,

    #[sqlx(rename = "tokenAddress")]
    pub token_address: Option<String>,

    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
