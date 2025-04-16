use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOpMessage {
    pub project_id: Option<String>,
    pub paymaster_mode: Option<String>,
    pub policy_id: Option<String>,
    pub token_address: Option<String>,
    pub status: String,
    pub data_source: Option<String>,
    pub timestamp: String,
    pub user_op: serde_json::Value,
    pub meta_data: Option<serde_json::Value>,

    #[serde(deserialize_with = "deserialize_lowercase")]
    pub user_op_hash: String,
}

fn deserialize_lowercase<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(s.to_lowercase())
}
