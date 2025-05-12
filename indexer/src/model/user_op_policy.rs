use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserOpPolicyData {
    pub policy_id: Option<String>,
    pub native_usd_price: Option<String>,
    pub actual_gas_cost: Option<String>,
    pub actual_gas_used: Option<String>,
    pub sender: Option<String>,
    pub enabled_limits: Option<Vec<String>>,
}

