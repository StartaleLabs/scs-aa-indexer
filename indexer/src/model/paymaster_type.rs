use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(sqlx::Type)]
#[sqlx(type_name = "paymaster_mode")]
pub enum PaymasterMode {
    #[serde(rename = "SPONSORSHIP_PREPAID")]
    SponsorshipPrepaid,
    #[serde(rename = "SPONSORSHIP_POSTPAID")]
    SponsorshipPostpaid,
    #[serde(rename = "TOKEN")]
    Token,
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl ToString for PaymasterMode {
    fn to_string(&self) -> String {
        match self {
            PaymasterMode::SponsorshipPrepaid => "SPONSORSHIP_PREPAID".to_string(),
            PaymasterMode::SponsorshipPostpaid => "SPONSORSHIP_POSTPAID".to_string(),
            PaymasterMode::Token => "TOKEN".to_string(),
            PaymasterMode::Unknown => "UNKNOWN".to_string(),
        }
    }
}

impl PaymasterMode {
    pub fn to_fund_type(&self) -> String {
        match self {
            PaymasterMode::SponsorshipPrepaid => "SELF_FUNDED".to_string(),
            PaymasterMode::SponsorshipPostpaid => "MANAGED".to_string(),
            PaymasterMode::Token => "USER_PAID".to_string(),
            PaymasterMode::Unknown => "UNKNOWN".to_string(),
        }
    }
}