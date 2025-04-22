// api/models/mod.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UserOperationRecord {
    pub id: i32,
    pub user_op_hash: String,
    pub user_operation: Option<serde_json::Value>,

    #[sqlx(rename = "policyid")]
    pub policy_id: Option<String>,

    #[sqlx(rename = "projectid")]
    pub project_id: Option<String>,

    #[sqlx(rename = "paymastermode")]
    pub paymaster_mode: Option<String>,

    #[sqlx(rename = "datasource")]
    pub data_source: Option<String>,

    pub status: String,

    #[sqlx(rename = "tokenaddress")]
    pub token_address: Option<String>,

    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
