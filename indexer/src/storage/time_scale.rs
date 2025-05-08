

use anyhow::Error;
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;
use crate::{consumer::kakfa_message::{UserOpMessage, Status}, storage::Storage};
use chrono::{DateTime, Utc};


#[derive(Clone)]
pub struct TimescaleStorage {
    pool: PgPool,
}

impl TimescaleStorage {
    pub async fn new(database_url: &str) -> Self {
        let pool = PgPool::connect(database_url).await.expect("Failed to connect to DB");
        Self { pool }
    }
}

impl TimescaleStorage {
    pub fn get_pg_pool(&self) -> &PgPool {
        &self.pool
    }
}
#[async_trait]
impl Storage for TimescaleStorage {
    async fn upsert_user_op_message(&self, msg: UserOpMessage) -> Result<(), Error> {
        let user_op_hash = msg.user_op_hash.trim();
        let event_time = msg.timestamp.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());
        let status_str = msg.status.to_string();
        let paymaster_mode = msg.paymaster_mode.as_ref().map(|m| m.to_string());

        tracing::info!("🟢 Upserting UserOpMessage with hash: {}", &user_op_hash);
        tracing::debug!("- useropmessage: {}", serde_json::to_string(&msg).unwrap_or_default());

        // Extract optional meta fields
        let (actual_gas_cost, actual_gas_used, deducted_user, deducted_amount, usd_amount) =
            msg.meta_data.as_ref().and_then(|v| v.as_object()).map_or(
                (None, None, None, None, None),
                |m| extract_meta_fields(m),
            );

        // Check if the entry exists already
        let existing: Option<String> = sqlx::query_scalar(
            "SELECT status FROM pm_user_operations WHERE user_op_hash = $1"
        )
        .bind(&user_op_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(current_status) = existing {
            let current_status = Status::from_str_case_insensitive(&current_status);
            let incoming_priority = msg.status.priority();
            let existing_priority = current_status.priority();

            tracing::info!("➡️ Comparing priorities: incoming={} vs existing={}", incoming_priority, existing_priority);

            if incoming_priority > existing_priority {
                // Higher priority update: update status + metadata + extracted fields
                let query = sqlx::query(
                    "UPDATE pm_user_operations 
                     SET status = $1, paymaster_mode = $2, data_source = $3,
                         metadata = metadata || $4::jsonb,
                         actual_gas_cost = $5, actual_gas_used = $6, deducted_user = $7,
                         deducted_amount = $8, usd_amount = $9
                     WHERE user_op_hash = $10"
                )
                .bind(&status_str)
                .bind(&paymaster_mode)
                .bind(&msg.data_source)
                .bind(Json(msg.meta_data))
                .bind(&actual_gas_cost)
                .bind(&actual_gas_used)
                .bind(&deducted_user)
                .bind(&deducted_amount)
                .bind(&usd_amount)
                .bind(user_op_hash);

                match query.execute(&self.pool).await {
                    Ok(res) if res.rows_affected() > 0 => {
                        tracing::info!("✅ Updated record with higher priority status ({})", status_str)
                    }
                    Ok(_) => tracing::warn!("⚠️ No rows updated despite higher priority."),
                    Err(e) => tracing::error!("❌ Failed to update record: {:?}", e),
                }
            } else {
                // Lower priority update: update auxiliary fields only
                let query = sqlx::query(
                    "UPDATE pm_user_operations 
                     SET owner_id = $1, paymaster_mode = $2, paymaster_id = $3, 
                         token_address = $4, data_source = $5, credential_id = $6
                     WHERE user_op_hash = $7"
                )
                .bind(&msg.owner_id)
                .bind(&paymaster_mode)
                .bind(&msg.paymaster_id)
                .bind(&msg.token_address)
                .bind(&msg.data_source)
                .bind(&msg.credential_id)
                .bind(user_op_hash);

                match query.execute(&self.pool).await {
                    Ok(_) => tracing::info!("📝 Updated metadata without changing status"),
                    Err(e) => tracing::error!("❌ Failed to update metadata: {:?}", e),
                }
            }
        } else {
            // Insert new record
            let query = sqlx::query(
                "INSERT INTO pm_user_operations 
                 (time, user_op_hash, user_operation, owner_id, credential_id, paymaster_mode, 
                  fund_type, paymaster_id, token_address, status, data_source, 
                  actual_gas_cost, actual_gas_used, deducted_user, deducted_amount, usd_amount, metadata) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)"
            )
            .bind(&event_time)
            .bind(&user_op_hash)
            .bind(&msg.user_op)
            .bind(&msg.owner_id)
            .bind(&msg.credential_id)
            .bind(&paymaster_mode)
            .bind(&msg.fund_type)
            .bind(&msg.paymaster_id)
            .bind(&msg.token_address)
            .bind(&status_str)
            .bind(&msg.data_source)
            .bind(&actual_gas_cost)
            .bind(&actual_gas_used)
            .bind(&deducted_user)
            .bind(&deducted_amount)
            .bind(&usd_amount)
            .bind(Json(msg.meta_data));

            match query.execute(&self.pool).await {
                Ok(_) => tracing::info!("✅ Inserted new record for hash {}", &user_op_hash),
                Err(e) => tracing::error!("❌ Failed to insert new record: {:?}", e),
            }
        }

        Ok(())
    }
}

fn extract_meta_fields(meta: &serde_json::Map<String, serde_json::Value>) -> (
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<f64>,
    Option<f64>,
) {
    let gas_cost = meta.get("actualGasCost")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let gas_used = meta.get("actualGasUsed")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<i64>().ok());

    let user = meta.get("deductedUser")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let deducted_amt = meta.get("deductedAmount")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    let usd_amt = meta.get("usdAmount")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok());

    (gas_cost, gas_used, user, deducted_amt, usd_amt)
}
