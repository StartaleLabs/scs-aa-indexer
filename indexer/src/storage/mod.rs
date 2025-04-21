pub mod time_scale;
use anyhow::Error;
use async_trait::async_trait;

use crate::consumer::kakfa_message::UserOpMessage;

#[async_trait]
pub trait Storage {
    async fn upsert_user_op_message(&self, msg: UserOpMessage) -> Result<(), Error>;

}