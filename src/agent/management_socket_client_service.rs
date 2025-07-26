use std::net::SocketAddr;
use std::sync::Arc;

use actix_web::rt;
use actix_web::web::Bytes;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::SinkExt as _;
use log::debug;
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

use crate::agent::receive_tokens_stopper_collection::ReceiveTokensStopperCollection;
use crate::agent::jsonrpc::Message as JsonRpcMessage;
use crate::agent::jsonrpc::Notification as JsonRpcNotification;
use crate::agent::jsonrpc::Request as JsonRpcRequest;
use crate::agent::jsonrpc::Response as JsonRpcResponse;
use crate::agent::receive_tokens_stopper_drop_guard::ReceiveTokensStopperDropGuard;
use crate::agent::jsonrpc::notification_params::SetStateParams;
use crate::agent::generate_tokens_request::GenerateTokensRequest;
use crate::agent::jsonrpc::notification_params::VersionParams;
use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::agent::slot_aggregated_status::SlotAggregatedStatus;
use crate::agent::from_request_params::FromRequestParams;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::Message as ManagementJsonRpcMessage;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::Notification as ManagementJsonRpcNotification;
use crate::agent::continue_conversation_request::ContinueConversationRequest;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::notification_params::RegisterAgentParams;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::notification_params::UpdateAgentStatusParams;
use crate::jsonrpc::Error as JsonRpcError;
use crate::jsonrpc::ResponseEnvelope;
use crate::jsonrpc::RequestEnvelope;
use crate::generated_token_envelope::GeneratedTokenEnvelope;
use crate::produces_snapshot::ProducesSnapshot;
use crate::jsonrpc::ErrorEnvelope;
use crate::service::Service;

pub struct ManagementSocketClientService {
    continue_conversation_request_tx: mpsc::UnboundedSender<ContinueConversationRequest>,
    generate_tokens_request_tx: mpsc::UnboundedSender<GenerateTokensRequest>,
    name: Option<String>,
    receive_tokens_stopper_collection: Arc<ReceiveTokensStopperCollection>,
    reconciliation_queue: Arc<ReconciliationQueue>,
    slot_aggregated_status: Arc<SlotAggregatedStatus>,
    socket_url: String,
}

impl ManagementSocketClientService {
    pub fn new(
        continue_conversation_request_tx: mpsc::UnboundedSender<ContinueConversationRequest>,
        generate_tokens_request_tx: mpsc::UnboundedSender<GenerateTokensRequest>,
        management_addr: SocketAddr,
        name: Option<String>,
        reconciliation_queue: Arc<ReconciliationQueue>,
        slot_aggregated_status: Arc<SlotAggregatedStatus>,
    ) -> Result<Self> {
        let agent_id = Uuid::new_v4();

        Ok(ManagementSocketClientService {
            continue_conversation_request_tx,
            generate_tokens_request_tx,
            name,
            receive_tokens_stopper_collection: Arc::new(ReceiveTokensStopperCollection::new()),
            reconciliation_queue,
            slot_aggregated_status,
            socket_url: format!("ws://{management_addr}/api/v1/agent_socket/{agent_id}"),
        })
    }

    async fn generate_tokens<TRequest: FromRequestParams + 'static>(
        connection_close_tx: broadcast::Sender<()>,
        id: String,
        message_tx: mpsc::UnboundedSender<ManagementJsonRpcMessage>,
        request_params: TRequest::RequestParams,
        receive_tokens_stopper_collection: Arc<ReceiveTokensStopperCollection>,
        request_tx: mpsc::UnboundedSender<TRequest>,
    ) -> Result<()> {
        let (generated_tokens_tx, mut generated_tokens_rx) =
            mpsc::unbounded_channel::<GeneratedTokenEnvelope>();
        let (generate_tokens_stop_tx, generate_tokens_stop_rx) = mpsc::unbounded_channel::<()>();

        let _guard = ReceiveTokensStopperDropGuard {
            receive_tokens_stopper_collection: receive_tokens_stopper_collection.clone(),
            request_id: id.clone(),
        };

        receive_tokens_stopper_collection
            .register_stopper(id.clone(), generate_tokens_stop_tx)
            .context(format!("Failed to register stopper for request ID: {id}"))?;

        request_tx.send(TRequest::from_request_params(
            request_params,
            generate_tokens_stop_rx,
            generated_tokens_tx,
        ))?;

        let mut connection_close_rx = connection_close_tx.subscribe();

        loop {
            tokio::select! {
                _ = connection_close_rx.recv() => break,
                generated_token_envelope = generated_tokens_rx.recv() => {
                    match generated_token_envelope {
                        Some(generated_token_envelope) => {
                            message_tx.send(
                                ManagementJsonRpcMessage::Response(
                                    ResponseEnvelope {
                                        request_id: id.clone(),
                                        response: JsonRpcResponse::GeneratedToken(generated_token_envelope),
                                    }
                                ),
                            )?;
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_deserialized_message(
        connection_close_tx: broadcast::Sender<()>,
        continue_conversation_request_tx: mpsc::UnboundedSender<ContinueConversationRequest>,
        generate_tokens_request_tx: mpsc::UnboundedSender<GenerateTokensRequest>,
        message_tx: mpsc::UnboundedSender<ManagementJsonRpcMessage>,
        deserialized_message: JsonRpcMessage,
        receive_tokens_stopper_collection: Arc<ReceiveTokensStopperCollection>,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<()> {
        match deserialized_message {
            JsonRpcMessage::Error(ErrorEnvelope {
                request_id,
                error: JsonRpcError { code, description },
            }) => {
                error!(
                    "Received error from server: code: {code}, description: {description:?}, request_id: {request_id:?}"
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
            JsonRpcMessage::Notification(JsonRpcNotification::StopGeneratingTokens(request_id)) => {
                debug!("Received StopGeneratingTokens notification for request ID: {request_id:?}");
                receive_tokens_stopper_collection
                    .stop(request_id.clone())
                    .context(format!(
                        "Failed to stop generating tokens for request ID: {request_id}"
                    ))?;

                Ok(())
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

                Ok(())
            }
            JsonRpcMessage::Request(RequestEnvelope {
                id,
                request: JsonRpcRequest::ContinueConversation(continue_conversation_params),
            }) => {
                Self::generate_tokens(
                    connection_close_tx,
                    id,
                    message_tx,
                    continue_conversation_params,
                    receive_tokens_stopper_collection,
                    continue_conversation_request_tx,
                )
                .await
            }
            JsonRpcMessage::Request(RequestEnvelope {
                id,
                request: JsonRpcRequest::GenerateTokens(generate_tokens_params),
            }) => {
                Self::generate_tokens(
                    connection_close_tx,
                    id,
                    message_tx,
                    generate_tokens_params,
                    receive_tokens_stopper_collection,
                    generate_tokens_request_tx,
                )
                .await
            }
        }
    }

    #[expect(clippy::too_many_arguments)]
    async fn handle_incoming_message(
        connection_close_tx: broadcast::Sender<()>,
        continue_conversation_request_tx: mpsc::UnboundedSender<ContinueConversationRequest>,
        generate_tokens_request_tx: mpsc::UnboundedSender<GenerateTokensRequest>,
        receive_tokens_stopper_collection: Arc<ReceiveTokensStopperCollection>,
        message_tx: mpsc::UnboundedSender<ManagementJsonRpcMessage>,
        msg: Message,
        pong_tx: mpsc::UnboundedSender<Bytes>,
        reconciliation_queue: Arc<ReconciliationQueue>,
    ) -> Result<()> {
        match msg {
            Message::Text(text) => {
                let mut connection_close_rx = connection_close_tx.subscribe();
                let receive_tokens_stopper_collection_clone =
                    receive_tokens_stopper_collection.clone();

                rt::spawn(async move {
                    tokio::select! {
                        _ = connection_close_rx.recv() => {
                            info!("Connection close signal received, shutting down");
                        }
                        result = Self::handle_deserialized_message(
                            connection_close_tx,
                            continue_conversation_request_tx,
                            generate_tokens_request_tx,
                            message_tx,
                            match serde_json::from_str::<JsonRpcMessage>(&text).context(format!("Failed to parse JSON-RPC message: {text}")) {
                                Ok(message) => message,
                                Err(err) => {
                                    error!("Failed to deserialize message: {err}");

                                    return;
                                }
                            },
                            receive_tokens_stopper_collection_clone,
                            reconciliation_queue,
                        ) => if let Err(err) = result {
                            error!("Error handling incoming message: {err}");
                        }
                    }
                });

                Ok(())
            }
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

        message_tx
            .send(ManagementJsonRpcMessage::Notification(
                ManagementJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                    name: self.name.clone(),
                    slot_aggregated_status_snapshot: self.slot_aggregated_status.make_snapshot(),
                }),
            ))
            .unwrap_or_else(|err| {
                error!("Failed to send register agent notification: {err}");
            });

        let do_send_status_update = || {
            let slot_aggregated_status_snapshot = self.slot_aggregated_status.make_snapshot();

            message_tx
                .send(ManagementJsonRpcMessage::Notification(
                    ManagementJsonRpcNotification::UpdateAgentStatus(UpdateAgentStatusParams {
                        slot_aggregated_status_snapshot,
                    }),
                ))
                .unwrap_or_else(|err| {
                    error!("Failed to send status update notification: {err}");
                });
        };

        let mut ticker = interval(Duration::from_secs(1));

        ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                _ = connection_close_rx.recv() => {
                    info!("Connection close signal received, shutting down");

                    break;
                }
                _ = shutdown.recv() => break,
                _ = self.slot_aggregated_status.update_notifier.notified() => do_send_status_update(),
                _ = ticker.tick() => do_send_status_update(),
                msg = read.next() => {
                    let should_close = match msg {
                        Some(Ok(msg)) => {
                            if let Err(err) = Self::handle_incoming_message(
                                    connection_close_tx.clone(),
                                    self.continue_conversation_request_tx.clone(),
                                    self.generate_tokens_request_tx.clone(),
                                    self.receive_tokens_stopper_collection.clone(),
                                    message_tx.clone(),
                                    msg,
                                    pong_tx.clone(),
                                    self.reconciliation_queue.clone(),
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
