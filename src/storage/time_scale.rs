

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
#[async_trait]
impl Storage for TimescaleStorage {
    async fn upsert_user_op_message(&self, msg: UserOpMessage) -> Result<(), Error> {
        let user_op_hash = msg.user_op_hash.trim();
        let created_at = msg.timestamp.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());
        let status_str = msg.status.to_string();

        println!("🟢 Upserting UserOpMessage with hash: {}", &user_op_hash);
        println!("- useropmessage: {}", serde_json::to_string(&msg).unwrap());

        let existing: Option<(i32, String)> = sqlx::query_as(
            "SELECT id, status FROM pm_user_operations WHERE user_op_hash = $1"
        )
        .bind(&user_op_hash)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((id, current_status)) = existing {
            let current_status = Status::from_str_case_insensitive(&current_status);
            let incoming_status = msg.status;
            
            let incoming_priority = incoming_status.priority();
            let existing_priority = current_status.priority();
            println!("➡️ incoming_priority: {}, existing_priority: {}", incoming_priority, existing_priority);
            if incoming_priority > existing_priority {
                // Update status + metadata if higher priority
                let query = if let Some(ref meta) = msg.meta_data {
                    sqlx::query(
                        "UPDATE pm_user_operations 
                         SET status = $1, paymasterMode = $2, dataSource = $3,
                             metadata = metadata || $4::jsonb, updated_at = now()
                         WHERE id = $5"
                    )
                    .bind(&status_str)
                    .bind(&msg.paymaster_mode)
                    .bind(&msg.data_source)
                    .bind(Json(meta))
                    .bind(id)
                } else {
                    sqlx::query(
                        "UPDATE pm_user_operations 
                         SET status = $1, paymasterMode = $2, dataSource = $3, 
                             updated_at = now() 
                         WHERE id = $4"
                    )
                    .bind(&status_str)
                    .bind(&msg.paymaster_mode)
                    .bind(&msg.data_source)
                    .bind(id)
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
                     SET policyId = $1, projectId = $2, paymasterMode = $3, dataSource = $4, tokenAddress = $5, updated_at = now() 
                     WHERE id = $6"
                )
                .bind(&msg.policy_id)
                .bind(&msg.project_id)
                .bind(&msg.paymaster_mode)
                .bind(&msg.data_source)
                .bind(&msg.token_address)
                .bind(id)
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
                 (user_op_hash, user_operation, policyId, projectId, paymasterMode, dataSource, status, tokenAddress, metadata, created_at, updated_at) 
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
            )
            .bind(&user_op_hash)
            .bind(&msg.user_op)
            .bind(&msg.policy_id)
            .bind(&msg.project_id)
            .bind(&msg.paymaster_mode)
            .bind(&msg.data_source)
            .bind(&status_str)
            .bind(&msg.token_address)
            .bind(Json(msg.meta_data.as_ref().unwrap_or(&serde_json::json!({}))))
            .bind(created_at)
            .bind(created_at)
            .execute(&self.pool)
            .await {
                Ok(_) => println!("✅ Inserted new record for hash {}", &user_op_hash),
                Err(e) => eprintln!("❌ Failed to insert new record: {:?}", e),
            }
        }

        Ok(())
    }
}

