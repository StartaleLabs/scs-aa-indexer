use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[derive(sqlx::Type)]
#[sqlx(type_name = "paymaster_mode", rename_all = "UPPERCASE")]
pub enum PaymasterMode {
    SponsorshipPrepaid,
    SponsorshipPostpaid,
    Token,
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

    // create a function to take paymastermode and return fundtype
    pub fn to_fund_type(&self) -> String {
        match self {
            PaymasterMode::SponsorshipPrepaid => "SELF".to_string(),
            PaymasterMode::SponsorshipPostpaid => "MANAGED".to_string(),
            PaymasterMode::Token => "USER_PAID".to_string(),
            PaymasterMode::Unknown => "UNKNOWN".to_string(),
        }
    }
}