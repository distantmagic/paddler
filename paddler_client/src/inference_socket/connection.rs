use std::sync::Arc;

use futures_util::StreamExt;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tokio_tungstenite::connect_async;
use url::Url;

use paddler_messaging::inference_client::message::Message as InferenceMessage;

use crate::error::Result;
use crate::inference_socket::cluster_inference_mode_broadcaster::ClusterInferenceModeBroadcaster;
use crate::inference_socket::inbound_message_router::InboundMessageRouter;
use crate::inference_socket::inference_socket_exchange::InferenceSocketExchange;
use crate::inference_socket::outbound_request::OutboundRequest;
use crate::inference_socket::spawn_read_task::spawn_read_task;
use crate::inference_socket::spawn_write_task::spawn_write_task;
use crate::inference_socket::url::url;

pub struct Connection {
    exchange: Arc<InferenceSocketExchange>,
    read_task: JoinHandle<()>,
    write_task: JoinHandle<()>,
}

impl Connection {
    pub async fn connect(
        connection_url: Url,
        cluster_inference_mode_broadcaster: Arc<ClusterInferenceModeBroadcaster>,
    ) -> Result<Self> {
        let ws_url = url(connection_url)?;
        let (ws_stream, _) = connect_async(ws_url.as_str()).await?;
        let (ws_write, ws_read) = ws_stream.split();
        let exchange = Arc::new(InferenceSocketExchange::default());

        Ok(Self {
            read_task: spawn_read_task(
                ws_read,
                InboundMessageRouter {
                    connection_generation: cluster_inference_mode_broadcaster.register_connection(),
                    cluster_inference_mode_broadcaster,
                    exchange: exchange.clone(),
                },
            ),
            write_task: spawn_write_task(ws_write, exchange.clone()),
            exchange,
        })
    }

    pub async fn disconnected(&self) {
        self.exchange.closed().await;
    }

    #[must_use]
    pub fn is_disconnected(&self) -> bool {
        self.exchange.is_closed()
    }

    pub fn send<TMessage: Serialize>(
        &self,
        request_id: String,
        message: &TMessage,
    ) -> Result<UnboundedReceiver<Result<InferenceMessage>>> {
        OutboundRequest::serialize(request_id, message)
            .and_then(|outbound_request| self.exchange.register(outbound_request))
    }

    pub fn stop_responding_to(&self, request_id: &str) {
        self.exchange.stop_responding_to(request_id);
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.read_task.abort();
        self.write_task.abort();
    }
}

#[cfg(test)]
mod tests {
    use std::io::ErrorKind;
    use std::sync::Arc;

    use tokio_tungstenite::tungstenite::Error as WebSocketError;
    use url::Url;

    use super::Connection;
    use crate::error::Error;
    use crate::inference_socket::cluster_inference_mode_broadcaster::ClusterInferenceModeBroadcaster;

    #[tokio::test]
    async fn connect_rejects_a_url_that_cannot_reach_an_inference_socket() {
        assert!(matches!(
            Connection::connect(
                Url::parse("ftp://127.0.0.1/models").expect("the test URL must be valid"),
                Arc::new(ClusterInferenceModeBroadcaster::default()),
            )
            .await,
            Err(Error::InferenceSocketUrlSchemeRejected { scheme, .. }) if scheme == "ftp"
        ));
    }

    #[tokio::test]
    async fn connect_fails_for_an_unreachable_server() {
        assert!(matches!(
            Connection::connect(
                Url::parse("http://127.0.0.1:1").expect("the test URL must be valid"),
                Arc::new(ClusterInferenceModeBroadcaster::default()),
            )
            .await,
            Err(Error::WebSocket(WebSocketError::Io(io_error)))
                if io_error.kind() == ErrorKind::ConnectionRefused
        ));
    }
}
