use redis::{AsyncCommands, RedisError};
use serde_json;
use crate::model::user_op_policy::UserOpPolicyData;
use crate::cache::Cache;
use anyhow::Error;
use async_trait::async_trait;
use crate::utils::{calculate_usd_spent, parse_gas_value, append_usage_update_cmds};

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
            let _: () = conn.set_ex(&key, serialized, 1800).await?;
        }

        Ok(())
    }

    async fn get_last_synced_block(
        &self,
        chain_id: u32,
    ) -> Result<Option<u64>, RedisError> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("sync_block:{}", chain_id);
        let block: Option<u64> = conn.get(key).await.ok();
        Ok(block)
    }

    async fn set_last_synced_block(
        &self,
        chain_id: u32,
        block_number: u64,
    ) -> Result<(), Error> {
        let mut conn = self.redis.get_async_connection().await?;
        let key = format!("sync_block:{}", chain_id);
        conn.set::<_, _, ()>(key, block_number).await.map_err(Error::from)?;
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
        let actual_gas_cost_str = data.actual_gas_cost.as_ref().unwrap();

        let usd_spent = calculate_usd_spent(actual_gas_cost_str, usd_price_str).unwrap_or(0.0);
        let gas = parse_gas_value(data.actual_gas_used.as_ref());

        let mut pipe = redis::pipe();

        if enabled.contains(&"GLOBAL".to_string()) {
            tracing::info!("🔄 Updating global usage limits");
            append_usage_update_cmds(&mut pipe, "global", policy_id, None, gas, usd_spent);
        }

        if enabled.contains(&"USER".to_string()) {
            tracing::info!("🔄 Updating user-specific usage limits");
            if let Some(user) = data.sender.as_ref() {
                append_usage_update_cmds(&mut pipe, "user", policy_id, Some(user), gas, usd_spent);
            }
        }

        let _: () = pipe.query_async(conn).await?;
        tracing::info!("🔄 Updated usage (scopes: {:?}): ops+=1 gas+={} usd+={:.4}", enabled, gas, usd_spent);
        Ok(())
    }
}
