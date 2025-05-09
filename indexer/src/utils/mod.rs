use sqlx::types::BigDecimal;
use std::str::FromStr;
use serde_json::{json, Value};

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
// Injects formatted USD amount into metadata and returns as BigDecimal (for Timescale use)
pub fn calculate_usd_amount_to_store(
    native_price: Option<BigDecimal>,
    actual_gas_cost_str: &str,
    meta_data: &mut Option<serde_json::Value>
) -> Option<BigDecimal> {
    if let (Some(price), true) = (native_price.clone(), !actual_gas_cost_str.is_empty()) {
        if let Some(usd) = calculate_usd_spent(actual_gas_cost_str, &price.to_string()) {
            if let Some(meta_map) = meta_data.as_mut().and_then(|v| v.as_object_mut()) {
                meta_map.insert("usdAmount".to_string(), json!(format!("{:.6}", usd)));
            }
            return BigDecimal::from_str(&format!("{:.6}", usd)).ok();
        }
    }
    None
}