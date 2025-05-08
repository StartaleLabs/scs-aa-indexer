// api/models/mod.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UserOperationRecord {
    pub time: DateTime<Utc>,
    pub user_op_hash: String,
    pub user_operation: Option<serde_json::Value>,
    pub paymaster_id: Option<String>,
    pub org_id: Option<String>,
    pub paymaster_mode: Option<String>,
    pub data_source: Option<String>,
    pub status: String,
    pub token: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub updated_at: NaiveDateTime,
}
