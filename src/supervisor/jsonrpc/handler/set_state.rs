use anyhow::Result;
use tokio::sync::mpsc::Sender;

use super::Handler;
use crate::supervisor::jsonrpc::request_params::SetStateParams;
use crate::supervisor::jsonrpc::Response as JsonRpcResponse;

pub struct SetState {}

impl SetState {
    pub fn new() -> Self {
        Self {}
    }
}

impl Handler<SetStateParams> for SetState {
    type ResponseResult = ();

    async fn handle(
        &self,
        sender: Sender<JsonRpcResponse<Self::ResponseResult>>,
        SetStateParams {
            request_id,
            desired_state,
        }: SetStateParams,
    ) -> Result<()> {
        Ok(())
    }
}
