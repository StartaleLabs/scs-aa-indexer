use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")] 
#[derive(sqlx::Type)]
#[sqlx(type_name = "paymaster_mode")]
pub enum PaymasterMode {
    Sponsorship,
    Token,
    #[serde(other)]
    Unknown,
}

impl ToString for PaymasterMode {
    fn to_string(&self) -> String {
        match self {
            PaymasterMode::Sponsorship => "SPONSORSHIP".to_string(),
            PaymasterMode::Token => "TOKEN".to_string(),
            PaymasterMode::Unknown => "UNKNOWN".to_string(),
        }
    }
}
