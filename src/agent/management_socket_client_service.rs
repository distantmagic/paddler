use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use log::error;
use log::info;
use log::warn;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use uuid::Uuid;

use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::jsonrpc::request_params::GenerateTokens as GenerateTokensParams;
use crate::agent::jsonrpc::Message as JsonRpcMessage;
use crate::agent::jsonrpc::Notification as JsonRpcNotification;
use crate::agent::jsonrpc::Request as JsonRpcRequest;
use crate::agent::message::GenerateTokens;
use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::agent::websocket_shared_writer::WebSocketSharedWriter;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::notification_params::RegisterAgentParams;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::Notification as ManagementJsonRpcNotification;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::RequestEnvelope;
use crate::jsonrpc::ResponseEnvelope;
use crate::service::Service;

pub struct ManagementSocketClientService {
    generate_tokens_tx: mpsc::Sender<GenerateTokens>,
    name: Option<String>,
    reconciliation_queue: Arc<ReconciliationQueue>,
    slots_total: usize,
    socket_url: String,
}

impl ManagementSocketClientService {
    pub fn new(
        generate_tokens_tx: mpsc::Sender<GenerateTokens>,
        management_addr: SocketAddr,
        name: Option<String>,
        reconciliation_queue: Arc<ReconciliationQueue>,
        slots_total: usize,
    ) -> Result<Self> {
        let agent_id = Uuid::new_v4();

        Ok(ManagementSocketClientService {
            generate_tokens_tx,
            name,
            reconciliation_queue,
            slots_total,
            socket_url: format!("ws://{management_addr}/api/v1/agent_socket/{agent_id}"),
        })
    }

    async fn handle_incoming_message(
        &self,
        msg: Message,
        writer: Arc<WebSocketSharedWriter>,
    ) -> Result<()> {
        match msg {
            Message::Text(text) => match serde_json::from_str::<JsonRpcMessage>(&text)
                .context(format!("Failed to parse JSON-RPC request: {text}"))?
            {
                JsonRpcMessage::Error(JsonRpcError {
                    code,
                    description,
                }) => {
                    error!(
                        "Received error from server: code: {code}, description: {description:?}"
                    );
                }
                JsonRpcMessage::Notification(JsonRpcNotification::SetState(SetStateParams {
                    desired_state,
                })) => {
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
                        JsonRpcRequest::GenerateTokens(GenerateTokensParams {
                            max_tokens,
                            prompt,
                        }),
                }) => {
                    let (chunk_sender, mut chunk_receiver) = mpsc::channel::<String>(100);
                    let writer_clone = writer.clone();

                    tokio::spawn(async move {
                        while let Some(chunk) = chunk_receiver.recv().await {
                            writer_clone
                                .send_serialized(ResponseEnvelope::StreamChunk {
                                    request_id: id.clone(),
                                    chunk,
                                })
                                .await?;
                        }

                        writer_clone
                            .send_serialized::<ResponseEnvelope<String>>(
                                ResponseEnvelope::StreamDone {
                                    request_id: id,
                                },
                            )
                            .await?;

                        Ok::<(), anyhow::Error>(())
                    });

                    let generate_tokens_tx = self.generate_tokens_tx.clone();

                    tokio::spawn(async move {
                        if let Err(err) = generate_tokens_tx
                            .send(GenerateTokens {
                                chunk_sender,
                                max_tokens,
                                prompt,
                            })
                            .await
                        {
                            error!("Failed to send GenerateTokens message: {err:?}");
                        }
                    });
                }
            },
            Message::Binary(_) => {
                error!("Received binary message, which is not expected");
            }
            Message::Close(_) => {
                info!("Connection closed by server");
            }
            Message::Frame(_) => {
                error!("Received a frame message, which is not expected");
            }
            Message::Ping(payload) => {
                writer.send(Message::Pong(payload)).await?;
            }
            Message::Pong(_) => {
                // Pong received, no action needed
            }
        }

        Ok(())
    }

    async fn keep_connection_alive(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let (ws_stream, _response) = connect_async(self.socket_url.clone()).await?;

        info!("Connected to management server");

        let (write, mut read) = ws_stream.split();
        let writer = Arc::new(WebSocketSharedWriter::new(write));

        writer
            .send_serialized(ManagementJsonRpcNotification::RegisterAgent(
                RegisterAgentParams {
                    name: self.name.clone(),
                    slots_total: self.slots_total,
                },
            ))
            .await?;

        loop {
            tokio::select! {
                _ = shutdown.recv() => {
                    info!("Shutdown signal received, closing connection");
                    writer.send_serialized(ManagementJsonRpcNotification::DeregisterAgent).await?;

                    break;
                }
                msg = read.next() => {
                    match msg {
                        Some(msg) => {
                            self.handle_incoming_message(msg?, writer.clone())
                                .await
                                .context("Failed to handle incoming message")?;
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl Service for ManagementSocketClientService {
    fn name(&self) -> &'static str {
        "agent::management_socket_client_service"
    }

    async fn run(&mut self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = shutdown.recv() => return Ok(()),
                _ = ticker.tick() => {
                    if let Err(err) = self.keep_connection_alive(shutdown.resubscribe()).await {
                        error!("Failed to keep the connection alive: {err:?}");
                    } else {
                        info!("Gracefully closed connection to management server");

                        return Ok(());
                    }
                }
            }
        }
    }
}
