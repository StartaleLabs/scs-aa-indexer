use redis::AsyncCommands;
use serde_json;
use crate::consumer::kakfa_message::UserOpPolicyData;
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
            println!("✅ Complete info for {}. Proceeding to update counters.", user_op_hash);
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
            println!("⚠️ Skipping update: no enabled limits specified.");
            return Ok(());
        };
        if enabled.is_empty() {
            println!("⚠️ Skipping update: enabled_limits is empty.");
            return Ok(());
        }

        let policy_id = data.policy_id.as_ref().unwrap();
        let usd = data.native_usd_price.unwrap();
        let actual_gas_cost = data.actual_gas_cost.as_ref().unwrap();

        let usd_spent = match actual_gas_cost.parse::<f64>() {
            Ok(cost_wei) => cost_wei * usd / 1e18,
            Err(_) => return Ok(()),
        };

        let gas = data.actual_gas_used
            .as_ref()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let mut pipe = redis::pipe();

        if enabled.contains(&"GLOBAL".to_string()) {
            let global_prefix = format!("global:{}", policy_id);
            pipe.cmd("INCRBY").arg(format!("{}:ops", global_prefix)).arg(1)
                .cmd("INCRBY").arg(format!("{}:gas", global_prefix)).arg(gas)
                .cmd("INCRBYFLOAT").arg(format!("{}:usd", global_prefix)).arg(usd_spent);
        }

        if enabled.contains(&"USER".to_string()) {
            if let Some(user) = data.sender.as_ref() {
                let user_prefix = format!("user:{}:{}", policy_id, user);
                pipe.cmd("INCRBY").arg(format!("{}:ops", user_prefix)).arg(1)
                    .cmd("INCRBY").arg(format!("{}:gas", user_prefix)).arg(gas)
                    .cmd("INCRBYFLOAT").arg(format!("{}:usd", user_prefix)).arg(usd_spent);
            }
        }

        let _: () = pipe.query_async(conn).await?;
        println!("🔄 Updated usage (scopes: {:?}): ops+=1 gas+={} usd+={:.4}", enabled, gas, usd_spent);
        Ok(())
    }
}
