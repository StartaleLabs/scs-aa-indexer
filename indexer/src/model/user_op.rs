use derive_more::derive::Display;
use serde::{Deserialize, Serialize};
use sqlx::{types::BigDecimal, FromRow};

use super::{fund_type::FundType, paymaster_mode::PaymasterMode};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOpMessage {
    pub org_id: Option<String>,
    pub credential_id : Option<String>,
    pub paymaster_mode: Option<PaymasterMode>,
    pub paymaster_id: Option<String>,
    pub policy_id: Option<String>,
    pub token_address: Option<String>,
    pub fund_type: Option<FundType>,
    pub chain_id: u32,
    pub status: Status,
    pub data_source: Option<String>,
    pub timestamp: String,
    pub user_op: serde_json::Value,
    pub meta_data: Option<serde_json::Value>,
    pub native_usd_price: Option<String>,
    pub user_op_hash: String,
    pub enabled_limits: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Status {
    Failed,
    Success,
    Eligible,
    #[serde(other)]
    Unknown,
}
#[derive(Debug, FromRow)]
pub struct UserOperationRecord {
    pub status: Option<String>,
    pub native_usd_price: Option<BigDecimal>,
    pub actual_gas_cost: Option<i64>,
    pub usd_amount: Option<BigDecimal>,
}

impl Status {
    pub fn from_str_case_insensitive(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "FAILED" => Status::Failed,
            "SUCCESS" => Status::Success,
            "ELIGIBLE" => Status::Eligible,
            _ => Status::Unknown,
        }
    }

    pub fn priority(&self) -> i32 {
        match self {
            Status::Failed => 3,
            Status::Success => 2,
            Status::Eligible => 1,
            Status::Unknown => 0,
        }
    }
}
