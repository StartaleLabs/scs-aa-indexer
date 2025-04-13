

use anyhow::Error;
use async_trait::async_trait;
use serde::Serialize;
use sqlx::PgPool;

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
        let existing: Option<(i64,)> = sqlx::query_as("SELECT id FROM pm_user_operations WHERE user_op_hash = $1")
            .bind(user_op_hash)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((id, )) = existing {
            sqlx::query(
                "UPDATE pm_user_operations SET status = $1, metadata = metadata || $2::jsonb, updated_at = now() WHERE id = $3"
            )
            .bind(&msg.status)
            .bind(&msg.meta_data)
            .bind(id)
            .execute(&self.pool)
            .await?;

        } else {
            sqlx::query(
                "INSERT INTO pm_user_operations (user_op_hash, user_operation, policyId, projectId, paymasterMode, dataSource, status, tokenAddress, metadata, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
            )
            .bind(&msg.user_op_hash)
            .bind(&msg.user_op)
            .bind(&msg.policy_id)
            .bind(&msg.project_id)
            .bind(&msg.paymaster_mode)
            .bind(&msg.data_source)
            .bind(&msg.status)
            .bind(&msg.token_address)
            .bind(serde_json::to_string(&msg.meta_data)?)
            .bind(created_at)
            .bind(created_at)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }
}
