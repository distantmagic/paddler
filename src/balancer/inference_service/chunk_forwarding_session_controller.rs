use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::session_controller::SessionController;
use crate::balancer::inference_service::http_route::api::ws_inference_socket::client::Message as OutgoingMessage;

pub struct ChunkForwardingSessionController {
    pub chunk_tx: mpsc::UnboundedSender<String>,
}

#[async_trait]
impl SessionController<OutgoingMessage> for ChunkForwardingSessionController {
    async fn send_response(&mut self, message: OutgoingMessage) -> anyhow::Result<()> {
        self.chunk_tx.send(serde_json::to_string(&message)?)?;

        Ok(())
    }
}
