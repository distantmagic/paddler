use std::net::SocketAddr;

use futures_util::SinkExt as _;
use futures_util::StreamExt as _;
use futures_util::future::ready;
use serde_json::from_str;
use serde_json::to_string;
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Error as WebSocketError;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;

use paddler_messaging::api_path::ApiPath;
use paddler_messaging::jsonrpc::request_envelope::RequestEnvelope;
use paddler_messaging::jsonrpc::response_envelope::ResponseEnvelope;
use paddler_messaging::management_socket::agent::message::Message as AgentJsonRpcMessage;
use paddler_messaging::management_socket::agent::request::Request as AgentJsonRpcRequest;
use paddler_messaging::management_socket::agent::response::Response as AgentJsonRpcResponse;
use paddler_messaging::management_socket::balancer::message::Message as ManagementJsonRpcMessage;
use paddler_messaging::management_socket::balancer::notification::Notification as ManagementJsonRpcNotification;
use paddler_messaging::management_socket::balancer::notification_params::register_agent_params::RegisterAgentParams;
use paddler_messaging::management_socket::balancer::notification_params::update_agent_status_params::UpdateAgentStatusParams;
use paddler_messaging::slot_aggregated_status_snapshot::SlotAggregatedStatusSnapshot;

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

    pub async fn connect_registered(
        management_addr: SocketAddr,
        agent_id: &str,
        slot_aggregated_status_snapshot: SlotAggregatedStatusSnapshot,
    ) -> Result<Self, ClusterHarnessError> {
        let mut untrusted_agent_socket_client = Self::connect(management_addr, agent_id)
            .await
            .map_err(ClusterHarnessError::AgentSocketConnectFailed)?;

        untrusted_agent_socket_client
            .register(RegisterAgentParams {
                name: Some(agent_id.to_owned()),
                slot_aggregated_status_snapshot,
            })
            .await?;

        Ok(untrusted_agent_socket_client)
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

    pub async fn register(
        &mut self,
        register_agent_params: RegisterAgentParams,
    ) -> Result<(), ClusterHarnessError> {
        self.send_management_message(&ManagementJsonRpcMessage::Notification(
            ManagementJsonRpcNotification::RegisterAgent(register_agent_params),
        ))
        .await
    }

    pub async fn report_status(
        &mut self,
        slot_aggregated_status_snapshot: SlotAggregatedStatusSnapshot,
    ) -> Result<(), ClusterHarnessError> {
        self.send_management_message(&ManagementJsonRpcMessage::Notification(
            ManagementJsonRpcNotification::UpdateAgentStatus(UpdateAgentStatusParams {
                slot_aggregated_status_snapshot,
            }),
        ))
        .await
    }

    pub async fn respond(
        &mut self,
        request_id: String,
        response: AgentJsonRpcResponse,
    ) -> Result<(), ClusterHarnessError> {
        self.send_management_message(&ManagementJsonRpcMessage::Response(ResponseEnvelope {
            generated_by: None,
            request_id,
            response,
        }))
        .await
    }

    pub async fn send(&mut self, message: Message) -> Result<(), ClusterHarnessError> {
        self.websocket
            .send(message)
            .await
            .map_err(ClusterHarnessError::AgentSocketSendFailed)
    }

    async fn send_management_message(
        &mut self,
        message: &ManagementJsonRpcMessage,
    ) -> Result<(), ClusterHarnessError> {
        self.send(Message::text(
            to_string(message).map_err(ClusterHarnessError::AgentSocketMessageUnserializable)?,
        ))
        .await
    }
}
