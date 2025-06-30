mod set_state;

use anyhow::Result;
use serde::Serialize;
use tokio::sync::mpsc::Sender;

pub use self::set_state::SetState;
use super::jsonrpc::request_params::RequestParams;
use super::jsonrpc::Response as JsonRpcResponse;

pub trait Handler<TRequestParams>
where
    TRequestParams: RequestParams,
{
    type ResponseResult: Serialize;

    async fn handle(
        &self,
        sender: Sender<JsonRpcResponse<Self::ResponseResult>>,
        params: TRequestParams,
    ) -> Result<()>;
}
