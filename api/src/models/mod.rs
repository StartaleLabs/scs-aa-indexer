// api/models/mod.rs
use serde::{Serialize, Deserialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc, NaiveDateTime};

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct UserOperationRecord {
    pub time: DateTime<Utc>,
    pub user_op_hash: String,
    pub user_operation: Option<serde_json::Value>,

    // Ownership and Paymaster Info
    pub paymaster_id: Option<String>,
    pub owner_id: Option<String>,
    pub paymaster_mode: Option<String>,
    pub data_source: Option<String>,
    pub status: String,
    pub actual_gas_cost: Option<i64>,
    pub actual_gas_used: Option<i64>,
    pub deducted_user: Option<String>,
    pub deducted_amount: Option<f64>,
    pub usd_amount: Option<f64>,
    pub token: Option<String>,
    pub premium: Option<f64>,
    pub token_charge: Option<f64>,
    pub applied_markup: Option<f64>,
    pub exchange_rate: Option<f64>,
    pub metadata: Option<serde_json::Value>,
    pub updated_at: NaiveDateTime,
}
