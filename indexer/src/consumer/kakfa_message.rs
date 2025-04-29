use derive_more::derive::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOpMessage {
    pub project_id: Option<String>,
    pub paymaster_mode: Option<String>,
    pub paymaster_id: Option<String>,
    pub token_address: Option<String>,
    pub fund_type: Option<String>,
    pub chain_id: Option<String>,
    pub status: Status,
    pub data_source: Option<String>,
    pub timestamp: String,
    pub user_op: serde_json::Value,
    pub meta_data: Option<serde_json::Value>,

    #[serde(deserialize_with = "deserialize_lowercase")]
    pub user_op_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Display)]
pub enum Status {
    Failed,
    Success,
    Eligible,
    #[serde(other)]
    Unknown,
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

fn deserialize_lowercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.to_lowercase())
}
