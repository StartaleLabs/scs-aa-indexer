use redis::AsyncCommands;
use serde_json;
use crate::model::user_op_policy::UserOpPolicyData;
use crate::cache::Cache;
use anyhow::Error;
use async_trait::async_trait;

pub struct RedisCoordinator {
    redis: redis::Client,
}

impl RedisCoordinator {
    pub fn new(redis_url: &str) -> Self {
        let client = redis::Client::open(redis_url).expect("Invalid Redis URL");
        Self { redis: client }
    }
}

#[async_trait]
impl Cache for RedisCoordinator {
    async fn update_userop_policy(
        &self,
        user_op_hash: &str,
        partial: UserOpPolicyData,
    ) -> Result<(), Error> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("userop:pending:{}", user_op_hash);

        tracing::info!("🟢 Updating Redis with key: {}", key);

        let existing: Option<String> = conn.get(&key).await?;
        let mut merged = if let Some(json_str) = existing {
            serde_json::from_str::<UserOpPolicyData>(&json_str).unwrap_or_default()
        } else {
            UserOpPolicyData::default()
        };

        // Merge fields
        if partial.enabled_limits.is_some() {
            merged.enabled_limits = partial.enabled_limits;
        }
        if partial.policy_id.is_some() {
            merged.policy_id = partial.policy_id;
        }
        if partial.native_usd_price.is_some() {
            merged.native_usd_price = partial.native_usd_price;
        }
        if partial.actual_gas_used.is_some() {
            merged.actual_gas_used = partial.actual_gas_used;
        }
        if partial.actual_gas_cost.is_some() {
            merged.actual_gas_cost = partial.actual_gas_cost;
        }
        if partial.sender.is_some() {
            merged.sender = partial.sender;
        }

        let is_complete = merged.policy_id.is_some()
            && merged.native_usd_price.is_some()
            && merged.actual_gas_cost.is_some()
            && merged.actual_gas_used.is_some();

        if is_complete {
            tracing::info!("✅ Complete info for {}. Proceeding to update counters.", user_op_hash);
            Self::update_usage_limits(&mut conn, &merged).await?;
            let _: () = conn.del(&key).await?;
        } else {
            let serialized = serde_json::to_string(&merged)?;
            let _: () = conn.set_ex(&key, serialized, 600).await?;
        }

        Ok(())
    }
}

impl RedisCoordinator {
    async fn update_usage_limits(
        conn: &mut redis::aio::Connection,
        data: &UserOpPolicyData,
    ) -> redis::RedisResult<()> {
        let Some(enabled) = data.enabled_limits.as_ref() else {
            tracing::info!("⚠️ Skipping update: no enabled limits specified.");
            return Ok(());
        };
        if enabled.is_empty() {
            tracing::info!("⚠️ Skipping update: enabled_limits is empty.");
            return Ok(());
        }
    
        let policy_id = data.policy_id.as_ref().unwrap();
        let usd_price_str = data.native_usd_price.as_ref().unwrap();
        let usd_price = match usd_price_str.parse::<f64>() {
            Ok(val) => val,
            Err(_) => {
                tracing::error!("❌ Failed to parse native_usd_price: {:?}", usd_price_str);
                return Ok(());
            }
        };
    
        let actual_gas_cost_str = data.actual_gas_cost.as_ref().unwrap();
        let cost_wei: f64 = match actual_gas_cost_str {
            s if s.starts_with("0x") => {
                u64::from_str_radix(s.trim_start_matches("0x"), 16)
                    .map(|v| v as f64)
                    .map_err(|e| e.to_string())
            }
            s => s.parse::<f64>().map_err(|e| e.to_string()),
        }
        .unwrap_or(0.0);
    
        let gas: u64 = match data.actual_gas_used.as_ref() {
            Some(val) if val.starts_with("0x") => {
                u64::from_str_radix(val.trim_start_matches("0x"), 16).unwrap_or(0)
            }
            Some(val) => val.parse::<u64>().unwrap_or(0),
            None => 0,
        };
    
        let usd_spent = cost_wei * usd_price / 1e18;
        
        let mut pipe = redis::pipe();
    
        if enabled.contains(&"GLOBAL".to_string()) {
            tracing::info!("🔄 Updating global usage limits");
            let global_prefix = format!("global:{}", policy_id);
            pipe.cmd("INCRBY").arg(format!("{}:ops", global_prefix)).arg(1)
                .cmd("INCRBY").arg(format!("{}:gas", global_prefix)).arg(gas)
                .cmd("INCRBYFLOAT").arg(format!("{}:usd", global_prefix))
                .arg(format!("{:.6}", usd_spent));
        }
    
        if enabled.contains(&"USER".to_string()) {
            tracing::info!("🔄 Updating user-specific usage limits");
            if let Some(user) = data.sender.as_ref() {
                let user_prefix = format!("user:{}:{}", policy_id, user);
                pipe.cmd("INCRBY").arg(format!("{}:ops", user_prefix)).arg(1)
                    .cmd("INCRBY").arg(format!("{}:gas", user_prefix)).arg(gas)
                    .cmd("INCRBYFLOAT").arg(format!("{}:usd", user_prefix))
                    .arg(format!("{:.6}", usd_spent));
            }
        }
    
        let _: () = pipe.query_async(conn).await?;
        tracing::info!("🔄 Updated usage (scopes: {:?}): ops+=1 gas+={} usd+={:.4}", enabled, gas, usd_spent);
        Ok(())
    }
    
}
