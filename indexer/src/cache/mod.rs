pub mod redis;
use anyhow::Error;
use async_trait::async_trait;

use crate::consumer::kakfa_message::UserOpPolicyData;

#[async_trait]
pub trait Cache {
    async fn update_userop_policy(&self, user_op_hash: &str, partial: UserOpPolicyData) -> Result<(), Error>;
}
