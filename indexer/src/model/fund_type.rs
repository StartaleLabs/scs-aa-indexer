use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] 
pub enum FundType {
    SelfFunded,
    Managed,
    #[serde(other)]
    Unknown,
}

impl ToString for FundType {
    fn to_string(&self) -> String {
        match self {
            FundType::SelfFunded => "SELF_FUNDED".to_string(),
            FundType::Managed => "MANAGED".to_string(),
            FundType::Unknown => "UNKNOWN".to_string(),
        }
    }
}
