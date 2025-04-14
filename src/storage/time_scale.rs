

use anyhow::Error;
use async_trait::async_trait;
use sqlx::PgPool;
use sqlx::types::Json;

use crate::{consumer::kakfa_message::UserOpMessage, storage::Storage};
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
        let user_op_hash = &msg.user_op_hash;
        let created_at = msg.timestamp.parse::<DateTime<Utc>>().unwrap_or_else(|_| Utc::now());
        println!("🟢 Upserting UserOpMessage with hash: {}", user_op_hash);
        println!("useropmessage: {}", serde_json::to_string(&msg).unwrap());

        // check if the record exists
        let existing: Option<(i64, String)> = sqlx::query_as(
            "SELECT id, status FROM pm_user_operations WHERE user_op_hash = $1"
        )
        .bind(user_op_hash)
        .fetch_optional(&self.pool)
        .await?;
        
            if let Some((id, current_status)) = existing {
                let incoming_priority = status_priority(&msg.status);
                let existing_priority = status_priority(&current_status);

                if incoming_priority > existing_priority {
                    let result = sqlx::query(
                        "UPDATE pm_user_operations SET status = $1, paymasterMode = $2, dataSource = $3, metadata = metadata || $4::jsonb, updated_at = now() WHERE id = $5"
                    )
                    .bind(&msg.status)
                    .bind(&msg.paymaster_mode)
                    .bind(&msg.data_source)
                    .bind(Json(&msg.meta_data))
                    .bind(id)
                    .execute(&self.pool)
                    .await;
                
                    match result {
                        Ok(_) => println!("✅ Updated record with higher priority status ({})", msg.status),
                        Err(e) => eprintln!("❌ Failed to update record: {:?}", e),
                    }
                } else {
                    let result = sqlx::query(
                        "UPDATE pm_user_operations SET policyId = $1, projectId = $2, paymasterMode = $3, dataSource = $4, tokenAddress = $5, updated_at = now() WHERE id = $6"
                    )
                    .bind(&msg.policy_id)
                    .bind(&msg.project_id)
                    .bind(&msg.paymaster_mode)
                    .bind(&msg.data_source)
                    .bind(&msg.token_address)
                    .bind(id)
                    .execute(&self.pool)
                    .await;
                
                    match result {
                        Ok(_) => println!("📝 Updated record metadata without changing status"),
                        Err(e) => eprintln!("❌ Failed to update metadata: {:?}", e),
                    }
                }                
            } else {
                let result = sqlx::query(
                    "INSERT INTO pm_user_operations (user_op_hash, user_operation, policyId, projectId, paymasterMode, dataSource, status, tokenAddress, metadata, created_at, updated_at) \
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)"
                )
                .bind(&msg.user_op_hash)
                .bind(&msg.user_op)
                .bind(&msg.policy_id)
                .bind(&msg.project_id)
                .bind(&msg.paymaster_mode)
                .bind(&msg.data_source)
                .bind(&msg.status)
                .bind(&msg.token_address)
                .bind(Json(&msg.meta_data))
                .bind(created_at)
                .bind(created_at)
                .execute(&self.pool)
                .await;
            
                match result {
                    Ok(_) => println!("✅ Inserted new record for hash {}", &msg.user_op_hash),
                    Err(e) => eprintln!("❌ Failed to insert new record: {:?}", e),
                }
            }
        Ok(())
    }
}

fn status_priority(status: &str) -> i32 {
    match status.to_uppercase().as_str() {
        "FAILED" => 3,
        "SUCCESS" => 2,
        "ELIGIBLE" => 1,
        _ => 0,
    }
}
