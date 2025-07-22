use std::net::SocketAddr;
use std::sync::Arc;

use actix_web::rt;
use actix_web::web::Bytes;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::SinkExt as _;
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

use crate::agent::jsonrpc::Message as JsonRpcMessage;
use crate::agent::jsonrpc::Notification as JsonRpcNotification;
use crate::agent::jsonrpc::Request as JsonRpcRequest;
use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::agent::slot_aggregated_status::SlotAggregatedStatus;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::Message as ManagementJsonRpcMessage;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::Notification as ManagementJsonRpcNotification;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::notification_params::RegisterAgentParams;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::notification_params::UpdateAgentStatusParams;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::RequestEnvelope;
use crate::produces_snapshot::ProducesSnapshot;
use crate::request_params::GenerateTokensParams;
use crate::service::Service;

pub struct ManagementSocketClientService {
    name: Option<String>,
    reconciliation_queue: Arc<ReconciliationQueue>,
    slot_aggregated_status: Arc<SlotAggregatedStatus>,
    socket_url: String,
}

impl ManagementSocketClientService {
    pub fn new(
        management_addr: SocketAddr,
        name: Option<String>,
        reconciliation_queue: Arc<ReconciliationQueue>,
        slot_aggregated_status: Arc<SlotAggregatedStatus>,
    ) -> Result<Self> {
        let agent_id = Uuid::new_v4();

        Ok(ManagementSocketClientService {
            name,
            reconciliation_queue,
            slot_aggregated_status,
            socket_url: format!("ws://{management_addr}/api/v1/agent_socket/{agent_id}"),
        })
    }

    async fn handle_incoming_message(
        _message_tx: mpsc::UnboundedSender<ManagementJsonRpcMessage>,
        msg: Message,
        pong_tx: mpsc::UnboundedSender<Bytes>,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<()> {
        match msg {
            Message::Text(text) => match serde_json::from_str::<JsonRpcMessage>(&text)
                .context(format!("Failed to parse JSON-RPC request: {text}"))?
            {
                JsonRpcMessage::Error(JsonRpcError { code, description }) => {
                    error!(
                        "Received error from server: code: {code}, description: {description:?}"
                    );

                    Ok(())
                }
                JsonRpcMessage::Notification(JsonRpcNotification::SetState(SetStateParams {
                    desired_state,
                })) => {
                    reconciliation_queue
                        .register_change_request(desired_state)
                        .await
                }
                JsonRpcMessage::Notification(JsonRpcNotification::StopRequest(_)) => Ok(()),
                JsonRpcMessage::Notification(JsonRpcNotification::Version(VersionParams {
                    version,
                })) => {
                    if version != env!("CARGO_PKG_VERSION") {
                        warn!(
                            "Version mismatch: server version is {version}, client version is {}",
                            env!("CARGO_PKG_VERSION")
                        );
                    }

                    Ok(())
                }
                JsonRpcMessage::Request(RequestEnvelope {
                    id: _,
                    request:
                        JsonRpcRequest::GenerateTokens(GenerateTokensParams {
                            max_tokens: _,
                            prompt: _,
                        }),
                }) => Ok(()),
            },
            Message::Binary(_) => {
                error!("Received binary message, which is not expected");

                Ok(())
            }
            Message::Close(_) => {
                info!("Connection closed by server");

                Ok(())
            }
            Message::Frame(_) => {
                error!("Received a frame message, which is not expected");

                Ok(())
            }
            Message::Ping(payload) => Ok(pong_tx.send(payload)?),
            Message::Pong(_) => {
                // Pong received, no action needed
                Ok(())
            }
        }
    }

    async fn keep_connection_alive(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        info!("Connecting to management server at {}", self.socket_url);

        let (ws_stream, _response) = connect_async(self.socket_url.clone()).await?;

        info!("Connected to management server");

        let (connection_close_tx, mut connection_close_rx) = broadcast::channel::<()>(1);
        let (message_tx, mut message_rx) = mpsc::unbounded_channel::<ManagementJsonRpcMessage>();
        let (pong_tx, mut pong_rx) = mpsc::unbounded_channel::<Bytes>();
        let (mut write, mut read) = ws_stream.split();

        let mut connection_close_rx_resubscribed = connection_close_rx.resubscribe();
        let mut shutdown_resubscribed = shutdown.resubscribe();

        let message_forward_handle = rt::spawn(async move {
            loop {
                tokio::select! {
                    _ = connection_close_rx_resubscribed.recv() => {
                        break;
                    }
                    _ = shutdown_resubscribed.recv() => {
                        info!("Shutdown signal received, deregistering agent");

                        write.send(Message::Text(match serde_json::to_string(
                            &ManagementJsonRpcMessage::Notification(
                                ManagementJsonRpcNotification::DeregisterAgent,
                            )
                        ) {
                            Ok(serialized_message) => serialized_message.into(),
                            Err(err) => {
                                error!("Failed to serialize deregister agent notification: {err}");
                                return;
                            }
                        })).await.unwrap_or_else(|err| {
                            error!("Failed to send deregister agent notification: {err}");
                        });

                        break;
                    }
                    message = message_rx.recv() => {
                        match message {
                            Some(msg) => {
                                match serde_json::to_string(&msg) {
                                    Ok(serialized_message) => {
                                        let message = Message::Text(serialized_message.into());

                                        if let Err(err) = write.send(message).await {
                                            error!("Failed to send message: {err}");
                                            break;
                                        }
                                    },
                                    Err(err) => {
                                        error!("Failed to serialize message: {err}");
                                    }
                                };
                            }
                            None => break,
                        }
                    }
                    payload = pong_rx.recv() => {
                        match payload {
                            Some(payload) => {
                                write.send(Message::Pong(payload)).await.unwrap_or_else(|err| {
                                    error!("Failed to send pong message: {err}");
                                });
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        let name_clone = self.name.clone();
        let reconciliation_queue_clone = self.reconciliation_queue.clone();
        let slot_aggregated_status_clone = self.slot_aggregated_status.clone();

        rt::spawn(async move {
            let mut ticker = interval(Duration::from_secs(1));

            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

            message_tx
                .send(ManagementJsonRpcMessage::Notification(
                    ManagementJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                        name: name_clone,
                        slot_aggregated_status_snapshot: slot_aggregated_status_clone.make_snapshot(),
                    }),
                ))
                .unwrap_or_else(|err| {
                    error!("Failed to send register agent notification: {err}");
                });

            let do_send_status_update = || {
                message_tx.send(
                    ManagementJsonRpcMessage::Notification(
                        ManagementJsonRpcNotification::UpdateAgentStatus(UpdateAgentStatusParams {
                            slot_aggregated_status_snapshot: slot_aggregated_status_clone.make_snapshot(),
                        })
                    )
                ).unwrap_or_else(|err| {
                    error!("Failed to send status update notification: {err}");
                });
            };

            loop {
                tokio::select! {
                    _ = connection_close_rx.recv() => {
                        info!("Connection close signal received, shutting down");

                        break;
                    }
                    _ = shutdown.recv() => break,
                    _ = slot_aggregated_status_clone.update_notifier.notified() => do_send_status_update(),
                    _ = ticker.tick() => do_send_status_update(),
                    msg = read.next() => {
                        let should_close = match msg {
                            Some(Ok(msg)) => {
                                if let Err(err) = Self::handle_incoming_message(
                                        message_tx.clone(),
                                        msg,
                                        pong_tx.clone(),
                                        reconciliation_queue_clone.clone(),
                                    )
                                    .await
                                    .context("Failed to handle incoming message")
                                {
                                    error!("Error handling incoming message: {err}");
                                }

                                false
                            }
                            Some(Err(err)) => {
                                error!("Error reading message: {err}");

                                true
                            }
                            None => true,
                        };

                        if should_close {
                            if let Err(err) = connection_close_tx.send(()) {
                                error!("Failed to send connection close signal: {err}");
                            }

                            break;
                        }
                    }
                }
            }
        })
        .await?;

        message_forward_handle
            .await
            .context("Failed to join message forwarding task")?;

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
                _ = shutdown.recv() => break Ok(()),
                _ = ticker.tick() => {
                    match self.keep_connection_alive(shutdown.resubscribe()).await {
                        Err(err) => {
                            error!("Failed to keep the connection alive: {err:?}");
                        }
                        Ok(()) => {
                            info!("Gracefully closed connection to management server");
                        }
                    }
                }
            }
        }
    }
}
