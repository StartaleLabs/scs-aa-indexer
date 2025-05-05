

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

        tracing::info!("🟢 Upserting UserOpMessage with hash: {}", &user_op_hash);
        tracing::debug!("- useropmessage: {}", serde_json::to_string(&msg).unwrap_or_default());

        let existing: Option<String> = sqlx::query_scalar(
            "SELECT status FROM pm_user_operations WHERE user_op_hash = $1"
        )
        .bind(&user_op_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(current_status) = existing {
            let current_status = Status::from_str_case_insensitive(&current_status);
            let incoming_status = msg.status;
            
            let incoming_priority = incoming_status.priority();
            let existing_priority = current_status.priority();
            let paymaster_mode: Option<String> = msg.paymaster_mode.as_ref().map(|m| m.to_string());

            println!("➡️ incoming_priority: {}, existing_priority: {}", incoming_priority, existing_priority);
            if incoming_priority > existing_priority {
                // Update status + metadata if higher priority

                let query = if let Some(ref meta) = msg.meta_data {
                    sqlx::query(
                        "UPDATE pm_user_operations 
                         SET status = $1, paymaster_mode = $2, data_source = $3,
                             metadata = metadata || $4::jsonb
                         WHERE user_op_hash = $5"
                    )
                    .bind(&status_str)
                    .bind(&paymaster_mode)
                    .bind(&msg.data_source)
                    .bind(Json(meta))
                    .bind(user_op_hash)
                } else {
                    sqlx::query(
                        "UPDATE pm_user_operations 
                         SET status = $1, paymaster_mode = $2, data_source = $3
                         WHERE user_op_hash = $4"
                    )
                    .bind(&status_str)
                    .bind(&paymaster_mode)                    
                    .bind(&msg.data_source)
                    .bind(user_op_hash)
                };

                match query.execute(&self.pool).await {
                    Ok(res) if res.rows_affected() > 0 => println!("✅ Updated record with higher priority status ({})", status_str),
                    Ok(_) => println!("⚠️ No rows updated despite higher priority."),
                    Err(e) => eprintln!("❌ Failed to update record: {:?}", e),
                }
            } else {
                // Just update metadata fields, not status
                match sqlx::query(
                    "UPDATE pm_user_operations 
                     SET project_id = $1, paymaster_mode = $2, paymaster_id = $3, token_address = $4, data_source = $5
                     WHERE user_op_hash = $6"
                )
                .bind(&msg.project_id)
                .bind(&paymaster_mode)    
                .bind(&msg.paymaster_id)
                .bind(&msg.token_address)
                .bind(&msg.data_source)
                .bind(user_op_hash)
                .execute(&self.pool)
                .await {
                    Ok(_) => println!("📝 Updated metadata without changing status"),
                    Err(e) => eprintln!("❌ Failed to update metadata: {:?}", e),
                }
            }
        } else {
            // Insert new record
            match sqlx::query(
                "INSERT INTO pm_user_operations 
                 (time, user_op_hash, user_operation, project_id, paymaster_mode, fund_type, paymaster_id, token_address, status, data_source, metadata) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
            )
            .bind(&event_time)
            .bind(&user_op_hash)
            .bind(&msg.user_op)
            .bind(&msg.project_id)
            .bind(&msg.paymaster_mode.as_ref().map(|m| m.to_string()))
            .bind(&msg.fund_type)
            .bind(&msg.paymaster_id)
            .bind(&msg.token_address)
            .bind(&status_str)
            .bind(&msg.data_source)
            .bind(Json(msg.meta_data.as_ref().unwrap_or(&serde_json::json!({}))))
            .execute(&self.pool)
            .await {
                Ok(_) => tracing::info!("✅ Inserted new record for hash {}", &user_op_hash),
                Err(e) => tracing::error!("❌ Failed to insert new record: {:?}", e),
            }
        }

        Ok(())
    }
}
