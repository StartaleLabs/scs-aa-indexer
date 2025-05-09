

use anyhow::Error;
use async_trait::async_trait;
use serde_json::json;
use sqlx::{types::BigDecimal, PgPool};
use sqlx::types::Json;
use crate::{model::user_op::{UserOpMessage, Status}, storage::Storage};
use chrono::{DateTime, Utc};
use crate::utils::{calculate_usd_spent, extract_meta_fields};
use std::str::FromStr;

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
    async fn upsert_user_op_message(&self, mut msg: UserOpMessage) -> Result<(), Error> {
        let chain_id = msg.chain_id as i32;
        let user_op_hash = msg.user_op_hash.trim();
        let event_time = msg.timestamp.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());
        let status_str = msg.status.to_string();
        let paymaster_mode = msg.paymaster_mode.as_ref().map(|m| m.to_string());

        tracing::info!("🟢 Upserting UserOpMessage with hash: {}", user_op_hash);
        tracing::debug!("- useropmessage: {}", serde_json::to_string(&msg).unwrap_or_default());

        // Step 1: Extract actualGasCost from metadata (as String)
        let actual_gas_cost_str = msg.meta_data
            .as_ref()
            .and_then(|m| m.get("actualGasCost"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Step 2: Query existing native_usd_price + actual_gas_cost from DB if needed
        let (db_native_price, db_actual_gas_cost): (Option<BigDecimal>, Option<i64>) = sqlx::query_as(
            "SELECT native_usd_price, actual_gas_cost FROM pm_user_operations WHERE user_op_hash = $1"
        )
        .bind(&user_op_hash)
        .fetch_optional(&self.pool)
        .await?
        .unwrap_or((None, None));

        // Step 3: Merge fallback values if needed
        let native_price = msg.native_usd_price
            .as_deref()
            .and_then(|s| BigDecimal::from_str(s).ok())
            .or(db_native_price);

        let actual_gas_cost_str = actual_gas_cost_str
            .or_else(|| db_actual_gas_cost.map(|v| v.to_string()))
            .unwrap_or_default();

        // Step 4: Backfill msg.native_usd_price if needed
        if msg.native_usd_price.is_none() {
            msg.native_usd_price = native_price.as_ref().map(|v| format!("{:.6}", v));
        }

        // Step 5: Inject usdAmount into metadata if possible
        let usd_amount_to_store = if let (Some(price), true) = (native_price.clone(), !actual_gas_cost_str.is_empty()) {
            if let Some(usd) = calculate_usd_spent(&actual_gas_cost_str, &price.to_string()) {
                if let Some(meta_map) = msg.meta_data.as_mut().and_then(|v| v.as_object_mut()) {
                    meta_map.insert("usdAmount".to_string(), json!(format!("{:.6}", usd)));
                }
                Some(BigDecimal::from_str(&format!("{:.6}", usd))?)
            } else {
                None
            }
        } else {
            None
        };

        // Step 6: Extract all metadata fields
        let (
            actual_gas_cost, actual_gas_used, deducted_user, deducted_amount, _,
            token, premium, token_charge, applied_markup, exchange_rate
        ) = msg.meta_data
            .as_ref()
            .and_then(|v| v.as_object())
            .map_or((None, None, None, None, None, None, None, None, None, None), |m| extract_meta_fields(m));

        // Step 7: Check if record exists
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
                sqlx::query(
                    "UPDATE pm_user_operations 
                     SET status = $1, paymaster_mode = $2, data_source = $3,
                         metadata = metadata || $4::jsonb,
                         actual_gas_cost = $5, actual_gas_used = $6, deducted_user = $7,
                         deducted_amount = $8, usd_amount = $9, token = $10,
                         premium = $11, token_charge = $12, applied_markup = $13, exchange_rate = $14,
                         native_usd_price = $15
                     WHERE chain_id = $16 AND user_op_hash = $17"
                )
                .bind(&status_str)
                .bind(&paymaster_mode)
                .bind(&msg.data_source)
                .bind(Json(msg.meta_data))
                .bind(&actual_gas_cost)
                .bind(&actual_gas_used)
                .bind(&deducted_user)
                .bind(&deducted_amount)
                .bind(&usd_amount_to_store)
                .bind(&token)
                .bind(&premium)
                .bind(&token_charge)
                .bind(&applied_markup)
                .bind(&exchange_rate)
                .bind(&native_price)
                .bind(&chain_id)
                .bind(user_op_hash)
                .execute(&self.pool)
                .await?;
            } else {
                sqlx::query(
                    "UPDATE pm_user_operations 
                     SET org_id = $1, paymaster_mode = $2, paymaster_id = $3, credential_id = $4
                     WHERE user_op_hash = $5"
                )
                .bind(&msg.org_id)
                .bind(&paymaster_mode)
                .bind(&msg.paymaster_id)
                .bind(&msg.credential_id)
                .bind(user_op_hash)
                .execute(&self.pool)
                .await?;
            }
        } else {
            sqlx::query(
                "INSERT INTO pm_user_operations 
                 (time, chain_id, user_op_hash, user_operation, org_id, credential_id, paymaster_mode, 
                  fund_type, paymaster_id, status, data_source, 
                  actual_gas_cost, actual_gas_used, deducted_user, deducted_amount, usd_amount, 
                  token, premium, token_charge, applied_markup, exchange_rate, native_usd_price, metadata) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,
                         $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23)"
            )
            .bind(&event_time)
            .bind(&chain_id)
            .bind(&user_op_hash)
            .bind(&msg.user_op)
            .bind(&msg.org_id)
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
            .bind(&usd_amount_to_store)
            .bind(&token)
            .bind(&premium)
            .bind(&token_charge)
            .bind(&applied_markup)
            .bind(&exchange_rate)
            .bind(&native_price)
            .bind(Json(msg.meta_data))
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}