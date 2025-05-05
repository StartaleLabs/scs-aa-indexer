use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(sqlx::Type)]
#[sqlx(type_name = "paymaster_mode", rename_all = "UPPERCASE")]
pub enum PaymasterMode {
    SPONSORSHIP,
    TOKEN,
    UNKNOWN,
}

impl ToString for PaymasterMode {
    fn to_string(&self) -> String {
        match self {
            PaymasterMode::SPONSORSHIP => "SPONSORSHIP".to_string(),
            PaymasterMode::TOKEN => "TOKEN".to_string(),
            PaymasterMode::UNKNOWN => "UNKNOWN".to_string(),
        }
    }
}