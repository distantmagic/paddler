use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::balancer::inference_service::http_route::api::ws_inference_socket::client::Message as OutgoingMessage;
use crate::controls_session::ControlsSession;

#[derive(Clone)]
pub struct ChunkForwardingSessionController {
    pub chunk_tx: mpsc::UnboundedSender<String>,
}

#[async_trait]
impl ControlsSession<OutgoingMessage> for ChunkForwardingSessionController {
    async fn send_response(&mut self, message: OutgoingMessage) -> anyhow::Result<()> {
        self.chunk_tx.send(serde_json::to_string(&message)?)?;

        Ok(())
    }
}
