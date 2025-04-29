// api/models/mod.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UserOperationRecord {
    pub id: i32,
    pub user_op_hash: String,
    pub user_operation: Option<serde_json::Value>,

    #[sqlx(rename = "paymaster_id")]
    pub paymaster_id: Option<String>,

    #[sqlx(rename = "project_id")]
    pub project_id: Option<String>,

    #[sqlx(rename = "paymaster_mode")]
    pub paymaster_mode: Option<String>,

    #[sqlx(rename = "data_source")]
    pub data_source: Option<String>,

    pub status: String,

    #[sqlx(rename = "token_address")]
    pub token_address: Option<String>,

    pub metadata: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
