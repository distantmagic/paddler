use anyhow::Result;
use serde::Serialize;
use tokio::sync::mpsc::Sender;

use super::request_params::RequestParams;
use super::Response as JsonRpcResponse;

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
