use serde::{Deserialize, Serialize};


#[derive(Debug, Serialize, Deserialize, Default)]
pub enum PaymasterType {
    SPONSORSHIP,
    TOKEN,
    #[default]
    UNKNOWN,
}