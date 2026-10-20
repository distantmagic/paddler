use std::net::SocketAddr;

use futures_util::SinkExt as _;
use futures_util::StreamExt as _;
use futures_util::future::ready;
use serde_json::from_str;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;

use paddler_messaging::api_path::ApiPath;
use paddler_messaging::jsonrpc::request_envelope::RequestEnvelope;
use paddler_messaging::management_socket::agent::message::Message as AgentJsonRpcMessage;
use paddler_messaging::management_socket::agent::request::Request as AgentJsonRpcRequest;

use crate::cluster_harness_error::ClusterHarnessError;

pub struct UntrustedAgentSocketClient {
    websocket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl UntrustedAgentSocketClient {
    pub async fn connect(
        management_addr: SocketAddr,
        agent_id: &str,
    ) -> Result<Self, WebSocketError> {
        connect_async(format!(
            "ws://{management_addr}{}",
            ApiPath::agent_socket(agent_id)
        ))
        .await
        .map(|(websocket, _handshake_response)| Self { websocket })
    }

    pub async fn next_close_frame(&mut self) -> Result<Option<CloseFrame>, ClusterHarnessError> {
        while let Some(frame) = self.websocket.next().await {
            if let Message::Close(close_frame) =
                frame.map_err(ClusterHarnessError::AgentSocketReceiveFailed)?
            {
                return Ok(close_frame);
            }
        }

        Err(ClusterHarnessError::AgentSocketEndedWithoutClosing)
    }

    pub async fn next_request(
        &mut self,
    ) -> Result<RequestEnvelope<AgentJsonRpcRequest>, ClusterHarnessError> {
        self.websocket
            .by_ref()
            .filter_map(|frame| {
                ready(
                    frame
                        .ok()
                        .and_then(|frame| frame.into_text().ok())
                        .and_then(|text| from_str::<AgentJsonRpcMessage>(&text).ok())
                        .and_then(|balancer_message| match balancer_message {
                            AgentJsonRpcMessage::Notification(_) => None,
                            AgentJsonRpcMessage::Request(request_envelope) => {
                                Some(request_envelope)
                            }
                        }),
                )
            })
            .next()
            .await
            .ok_or(ClusterHarnessError::AgentSocketEndedWithoutRequest)
    }

    pub async fn send(&mut self, message: Message) -> Result<(), ClusterHarnessError> {
        self.websocket
            .send(message)
            .await
            .map_err(ClusterHarnessError::AgentSocketSendFailed)
    }
}
