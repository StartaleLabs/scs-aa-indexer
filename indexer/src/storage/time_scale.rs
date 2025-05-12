

use anyhow::Error;
use async_trait::async_trait;
use sqlx::{types::BigDecimal, PgPool};
use crate::{model::user_op::{UserOpMessage, Status, UserOperationRecord}, storage::Storage};
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

        let existing: Option<UserOperationRecord> = sqlx::query_as::<_, UserOperationRecord>(
            "SELECT status, native_usd_price, actual_gas_cost, usd_amount \
             FROM pm_user_operations \
             WHERE user_op_hash = $1"
        )
        .bind(&user_op_hash)
        .fetch_optional(&self.pool)
        .await?;

        let db_native_price = existing.as_ref().and_then(|e| e.native_usd_price.clone());
        let db_actual_gas_cost = existing.as_ref().and_then(|e| e.actual_gas_cost);

        let actual_gas_cost_str = msg.meta_data
            .as_ref()
            .and_then(|m| m.get("actualGasCost"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| db_actual_gas_cost.map(|v| v.to_string()));

        let native_price = msg.native_usd_price
            .as_deref()
            .and_then(|s| BigDecimal::from_str(s).ok())
            .or(db_native_price);

        if msg.native_usd_price.is_none() {
            msg.native_usd_price = native_price.as_ref().map(|v| format!("{:.6}", v));
        }

        let usd_amount_to_store = calculate_usd_spent(
            native_price.as_ref().map(|v| v.to_string()).as_deref().unwrap_or(""),
            actual_gas_cost_str.as_deref().unwrap_or("")
        ).and_then(|s| BigDecimal::from_str(&s.to_string()).ok());

        let (
            actual_gas_cost, actual_gas_used, deducted_user, deducted_amount,
            token, premium, token_charge, applied_markup, exchange_rate
        ) = msg.meta_data
            .as_ref()
            .and_then(|v| v.as_object())
            .map_or((None, None, None, None, None, None, None, None, None), |m| extract_meta_fields(m));

        // Heuristic for account deployment: assumes if either `factory` or `factoryData` is present, deployment was intended.
        let account_deployed = msg.user_op.get("factory")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty() && s != "0x")
            .unwrap_or(false)
            ||
            msg.user_op.get("factoryData")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty() && s != "0x")
            .unwrap_or(false);

        if let Some(e) = existing {
            let current_status = Status::from_str_case_insensitive(e.status.as_deref().unwrap_or_default());
            let incoming_priority = msg.status.priority();
            let existing_priority = current_status.priority();

            let coalesced_usd_amount = usd_amount_to_store.or(e.usd_amount);
            let coalesced_native_price = native_price.or(e.native_usd_price.clone());

            let update_query = sqlx::query(
                "UPDATE pm_user_operations \
                 SET status = $1, paymaster_mode = $2, data_source = $3,\
                     metadata = metadata || $4::jsonb,\
                     actual_gas_cost = $5, actual_gas_used = $6, deducted_user = $7,\
                     deducted_amount = $8, usd_amount = $9, token = $10,\
                     premium = $11, token_charge = $12, applied_markup = $13, exchange_rate = $14 \
                 WHERE user_op_hash = $15"
            )
            .bind(&status_str)
            .bind(&paymaster_mode)
            .bind(&msg.data_source)
            .bind(msg.meta_data.as_ref().unwrap_or(&serde_json::Value::Null))
            .bind(&actual_gas_cost)
            .bind(&actual_gas_used)
            .bind(&deducted_user)
            .bind(&deducted_amount)
            .bind(&coalesced_usd_amount)
            .bind(&token)
            .bind(&premium)
            .bind(&token_charge)
            .bind(&applied_markup)
            .bind(&exchange_rate)
            .bind(&user_op_hash);

            if incoming_priority > existing_priority {
                update_query.execute(&self.pool).await?;
            } else {
                sqlx::query(
                    "UPDATE pm_user_operations
                    SET org_id = COALESCE(org_id, $1),
                        paymaster_mode = COALESCE(paymaster_mode,$2),
                        paymaster_id = COALESCE(paymaster_id, $3),
                        credential_id = COALESCE(credential_id, $4),
                        metadata = metadata || $5::jsonb,
                        usd_amount = COALESCE(usd_amount, $6),
                        native_usd_price = COALESCE(native_usd_price, $7),
                        account_deployed = COALESCE(account_deployed, $8),
                        fund_type = COALESCE(fund_type, $9)
                    WHERE user_op_hash = $10"
                )
                .bind(&msg.org_id)
                .bind(&paymaster_mode)
                .bind(&msg.paymaster_id)
                .bind(&msg.credential_id)
                .bind(msg.meta_data.as_ref().unwrap_or(&serde_json::Value::Null))
                .bind(&coalesced_usd_amount)
                .bind(&coalesced_native_price)
                .bind(&account_deployed)
                .bind(&msg.fund_type)
                .bind(&user_op_hash)
                .execute(&self.pool)
                .await?;
            }
        } else {
            sqlx::query(
                "INSERT INTO pm_user_operations \
                 (time, chain_id, user_op_hash, user_operation, org_id, credential_id, paymaster_mode, \
                  fund_type, paymaster_id, status, data_source, \
                  actual_gas_cost, actual_gas_used, deducted_user, deducted_amount, usd_amount, \
                  token, premium, token_charge, applied_markup, exchange_rate, native_usd_price, metadata, account_deployed) \
                 VALUES (\
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11,\
                    $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24\
                 )"
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
            .bind(msg.meta_data.as_ref().unwrap_or(&serde_json::Value::Null))
            .bind(&account_deployed)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
