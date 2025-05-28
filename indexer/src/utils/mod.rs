use sqlx::types::BigDecimal;
use std::str::FromStr;
use serde_json::Value;

// Converts gas cost string + native token USD price into a `f64` USD amount
pub fn calculate_usd_spent(actual_gas_cost: &str, native_usd_price: &str) -> Option<f64> {
    let cost_wei = if actual_gas_cost.starts_with("0x") {
        u64::from_str_radix(actual_gas_cost.trim_start_matches("0x"), 16).ok()? as f64
    } else {
        actual_gas_cost.parse::<f64>().ok()?
    };

    let usd_price = native_usd_price.parse::<f64>().ok()?;
    Some(cost_wei * usd_price / 1e18)
}

pub fn parse_gas_value(val: Option<&String>) -> u64 {
    match val {
        Some(s) if s.starts_with("0x") => u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0),
        Some(s) => s.parse::<u64>().unwrap_or(0),
        None => 0,
    }
}

pub fn extract_meta_fields(meta: &serde_json::Map<String, Value>) -> (
    Option<i64>,               // actualGasCost
    Option<i64>,               // actualGasUsed
    Option<String>,            // deductedUser
    Option<BigDecimal>,        // deductedAmount
    Option<String>,            // token
    Option<BigDecimal>,        // premium
    Option<BigDecimal>,        // tokenCharge
    Option<BigDecimal>,        // appliedMarkup
    Option<BigDecimal>         // exchangeRate
) {
    let get_str = |key: &str| meta.get(key).and_then(|v| v.as_str());
    let parse_i64 = |key: &str| get_str(key).and_then(|s| s.parse::<i64>().ok());
    let parse_str = |key: &str| get_str(key).map(|s| s.to_string());
    let parse_decimal = |key: &str| get_str(key).and_then(|s| BigDecimal::from_str(s).ok());

    (
        parse_i64("actualGasCost"),
        parse_i64("actualGasUsed"),
        parse_str("deductedUser"),
        parse_decimal("deductedAmount"),
        parse_str("token"),
        parse_decimal("premium"),
        parse_decimal("tokenCharge"),
        parse_decimal("appliedMarkup"),
        parse_decimal("exchangeRate"),
    )
}

pub fn append_usage_update_cmds(
    pipe: &mut redis::Pipeline,
    scope: &str,
    policy_id: &str,
    user: Option<&str>,
    gas: u64,
    usd_spent: f64,
) {
    let prefix = match user {
        Some(u) => format!("{}:{}:{}", scope, policy_id, u),
        None => format!("{}:{}", scope, policy_id),
    };

    // Increment confirmed usage counters
    pipe.cmd("INCRBY").arg(format!("{}:ops", &prefix)).arg(1)
        .cmd("INCRBY").arg(format!("{}:gas", &prefix)).arg(gas)
        .cmd("INCRBYFLOAT").arg(format!("{}:usd", &prefix)).arg(format!("{:.6}", usd_spent));

    // Delete pending buffer keys
    pipe.cmd("DEL")
        .arg(format!("{}:pending_ops", &prefix))
        .arg(format!("{}:pending_gas", &prefix))
        .arg(format!("{}:pending_usd", &prefix));
}
