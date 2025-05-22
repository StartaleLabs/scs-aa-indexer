pub mod redis;
use anyhow::Error;
use async_trait::async_trait;
use ::redis::RedisError;

use crate::model::user_op_policy::UserOpPolicyData;
#[async_trait]
pub trait Cache {
    async fn get_last_synced_block(&self, chain_id : u32) -> Result<Option<u64>, RedisError>;
    async fn set_last_synced_block(&self, chain_id : u32, block_number: u64) -> Result<(), Error>;
    async fn update_userop_policy(&self, user_op_hash: &str, partial: UserOpPolicyData) -> Result<(), Error>;
}
