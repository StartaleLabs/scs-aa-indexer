

use anyhow::Error;
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;
use crate::{model::user_op::{UserOpMessage, Status}, storage::Storage};
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
        let chain_id = msg.chain_id as i32;
        let user_op_hash = msg.user_op_hash.trim();
        let event_time = msg.timestamp.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());
        let status_str = msg.status.to_string();
        let paymaster_mode = msg.paymaster_mode.as_ref().map(|m| m.to_string());

        tracing::info!("🟢 Upserting UserOpMessage with hash: {}", &user_op_hash);
        tracing::debug!("- useropmessage: {}", serde_json::to_string(&msg).unwrap_or_default());

        // Extract optional metadata fields
        let (
            actual_gas_cost, actual_gas_used, deducted_user, deducted_amount, usd_amount,
            token, premium, token_charge, applied_markup, exchange_rate
        ) = msg.meta_data
            .as_ref()
            .and_then(|v| v.as_object())
            .map_or((None, None, None, None, None, None, None, None, None, None), |m| extract_meta_fields(m));

        // Check if the entry exists
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

            if incoming_priority > existing_priority {
                let query = sqlx::query(
                    "UPDATE pm_user_operations 
                     SET status = $1, paymaster_mode = $2, data_source = $3,
                         metadata = metadata || $4::jsonb,
                         actual_gas_cost = $5, actual_gas_used = $6, deducted_user = $7,
                         deducted_amount = $8, usd_amount = $9, token = $10,
                         premium = $11, token_charge = $12, applied_markup = $13, exchange_rate = $14
                     WHERE chain_id = $16 AND user_op_hash = $15"
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
                .bind(&token)
                .bind(&premium)
                .bind(&token_charge)
                .bind(&applied_markup)
                .bind(&exchange_rate)
                .bind(&chain_id)
                .bind(user_op_hash);

                match query.execute(&self.pool).await {
                    Ok(res) if res.rows_affected() > 0 => tracing::info!("✅ Updated record with higher priority status ({})", status_str),
                    Ok(_) => tracing::warn!("⚠️ No rows updated despite higher priority."),
                    Err(e) => tracing::error!("❌ Failed to update record: {:?}", e),
                }
            } else {
                let query = sqlx::query(
                    "UPDATE pm_user_operations 
                     SET org_id = $1, paymaster_mode = $2, paymaster_id = $3, 
                         data_source = $4, credential_id = $5
                     WHERE user_op_hash = $6"
                )
                .bind(&msg.owner_id)
                .bind(&paymaster_mode)
                .bind(&msg.paymaster_id)
                .bind(&msg.data_source)
                .bind(&msg.credential_id)
                .bind(user_op_hash);

                match query.execute(&self.pool).await {
                    Ok(_) => tracing::info!("📝 Updated metadata without changing status"),
                    Err(e) => tracing::error!("❌ Failed to update metadata: {:?}", e),
                }
            }
        } else {
            let query = sqlx::query(
                "INSERT INTO pm_user_operations 
                 (time, user_op_hash, user_operation, org_id, credential_id, paymaster_mode, 
                  fund_type, paymaster_id, status, data_source, 
                  actual_gas_cost, actual_gas_used, deducted_user, deducted_amount, usd_amount, 
                  token, premium, token_charge, applied_markup, exchange_rate, metadata) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                         $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)"
            )
            .bind(&event_time)
            .bind(&user_op_hash)
            .bind(&msg.user_op)
            .bind(&msg.owner_id)
            .bind(&msg.credential_id)
            .bind(&paymaster_mode)
            .bind(&msg.fund_type)
            .bind(&msg.paymaster_id)
            .bind(&status_str)
            .bind(&msg.data_source)
            .bind(&actual_gas_cost)
            .bind(&actual_gas_used)
            .bind(&deducted_user)
            .bind(&deducted_amount)
            .bind(&usd_amount)
            .bind(&token)
            .bind(&premium)
            .bind(&token_charge)
            .bind(&applied_markup)
            .bind(&exchange_rate)
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
    Option<i64>, Option<i64>, Option<String>, Option<f64>, Option<f64>,
    Option<String>, Option<f64>, Option<f64>, Option<f64>, Option<f64>
) {
    let get_str = |key: &str| meta.get(key).and_then(|v| v.as_str());
    let parse_i64 = |key: &str| get_str(key).and_then(|s| s.parse::<i64>().ok());
    let parse_f64 = |key: &str| get_str(key).and_then(|s| s.parse::<f64>().ok());
    let parse_str = |key: &str| get_str(key).map(|s| s.to_string());

    (
        parse_i64("actualGasCost"),
        parse_i64("actualGasUsed"),
        parse_str("deductedUser"),
        parse_f64("deductedAmount"),
        parse_f64("usdAmount"),
        parse_str("token"),
        parse_f64("premium"),
        parse_f64("tokenCharge"),
        parse_f64("appliedMarkup"),
        parse_f64("exchangeRate"),
    )
}
