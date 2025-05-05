pub mod redis;
use anyhow::Error;
use async_trait::async_trait;

use crate::model::user_op_policy::UserOpPolicyData;
#[async_trait]
pub trait Cache {
    async fn update_userop_policy(&self, user_op_hash: &str, partial: UserOpPolicyData) -> Result<(), Error>;
}
