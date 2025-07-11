use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::SinkExt;
use futures_util::StreamExt;
use log::debug;
use log::error;
use log::info;
use log::warn;
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::jsonrpc::request_params::GenerateTokens;
use crate::agent::jsonrpc::Message as JsonRpcMessage;
use crate::agent::jsonrpc::Notification as JsonRpcNotification;
use crate::agent::jsonrpc::Request as JsonRpcRequest;
use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::agent::websocket_shared_writer::WebSocketSharedWriter;
use crate::balancer::http_route::api::ws_agent::jsonrpc::notification_params::RegisterAgentParams;
use crate::balancer::http_route::api::ws_agent::jsonrpc::Notification as ManagementJsonRpcNotification;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::RequestEnvelope;
use crate::service::Service;

pub struct ManagementSocketClientService {
    name: Option<String>,
    reconciliation_queue: Arc<ReconciliationQueue>,
    socket_url: String,
}

impl ManagementSocketClientService {
    pub fn new(
        management_addr: SocketAddr,
        name: Option<String>,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<Self> {
        let agent_id = Uuid::new_v4();

        Ok(ManagementSocketClientService {
            name,
            reconciliation_queue,
            socket_url: format!("ws://{management_addr}/api/v1/agent_socket/{agent_id}"),
        })
    }

    async fn keep_connection_alive(&self) -> Result<()> {
        let (ws_stream, _response) = connect_async(self.socket_url.clone()).await?;

        info!("Connected to management server");

        let (mut write, mut read) = ws_stream.split();
        let writer = WebSocketSharedWriter::new(write);

        writer
            .send(Message::Text(
                serde_json::to_string(&ManagementJsonRpcNotification::RegisterAgent(
                    RegisterAgentParams {
                        name: self.name.clone(),
                    },
                ))
                .context("Failed to serialize RegisterAgent notification")?
                .into(),
            ))
            .await?;

        while let Some(msg) = read.next().await {
            match msg? {
                Message::Text(text) => match serde_json::from_str::<JsonRpcMessage>(&text)
                    .context(format!("Failed to parse JSON-RPC request: {text}"))?
                {
                    JsonRpcMessage::Error(JsonRpcError {
                        code,
                        description,
                    }) => {
                        error!("Received error from server: code: {code}, description: {description:?}");
                    }
                    JsonRpcMessage::Notification(JsonRpcNotification::SetState(
                        SetStateParams {
                            desired_state,
                        },
                    )) => {
                        self.reconciliation_queue
                            .register_change_request(desired_state)
                            .await?;
                    }
                    JsonRpcMessage::Notification(JsonRpcNotification::Version(VersionParams {
                        version,
                    })) => {
                        if version != env!("CARGO_PKG_VERSION") {
                            warn!(
                                "Version mismatch: server version is {version}, client version is {}",
                                env!("CARGO_PKG_VERSION")
                            );
                        }
                    }
                    JsonRpcMessage::Request(RequestEnvelope {
                        id,
                        request:
                            JsonRpcRequest::GenerateTokens(GenerateTokens {
                                prompt,
                            }),
                    }) => {
                        println!("Received GenerateTokens request with id, prompt: {id}, {prompt}");
                    }
                },
                Message::Binary(_) => {
                    error!("Received binary message, which is not expected");
                }
                Message::Close(_) => {
                    info!("Connection closed by server");
                    break;
                }
                Message::Ping(payload) => {
                    writer.send(Message::Pong(payload)).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Service for ManagementSocketClientService {
    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    debug!("Shutting down monitoring service");
                    return Ok(());
                },
                _ = ticker.tick() => {
                    if let Err(err) = self.keep_connection_alive().await {
                        error!("Failed to keep the connection alive: {err:?}");
                    }
                }
            }
        }
    }
}
