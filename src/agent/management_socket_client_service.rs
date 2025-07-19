use std::net::SocketAddr;
use futures_util::SinkExt as _;
use actix_web::rt;
use std::sync::Arc;
use crate::request_params::GenerateTokensParams;
use anyhow::Context;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt;
use log::error;
use actix_web::web::Bytes;
use crate::agent::generate_tokens_stoppers_collection::GenerateTokensStoppersCollection;
use log::info;
use std::sync::atomic::Ordering::Relaxed;
use log::warn;
use crate::agent::generate_tokens_stop_result::GenerateTokensStopResult;
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
use crate::agent::jsonrpc::Message as JsonRpcMessage;
use crate::agent::jsonrpc::Notification as JsonRpcNotification;
use crate::agent::jsonrpc::Request as JsonRpcRequest;
use crate::agent::jsonrpc::Response as JsonRpcResponse;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::notification_params::RegisterAgentParams;
use crate::agent::message::GenerateTokensChannel;
use crate::response::ChunkResponse;
use crate::agent::reconciliation_queue::ReconciliationQueue;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::Notification as ManagementJsonRpcNotification;
use crate::balancer::management_service::http_route::api::ws_agent_socket::jsonrpc::Message as ManagementJsonRpcMessage;
use crate::jsonrpc::Error as JsonRpcError;
use crate::agent::slot_aggregated_metrics::SlotAggregatedMetrics;
use crate::jsonrpc::RequestEnvelope;
use crate::jsonrpc::ResponseEnvelope;
use crate::service::Service;

pub struct ManagementSocketClientService {
    generate_tokens_channel_tx: mpsc::Sender<GenerateTokensChannel>,
    generate_tokens_stoppers_collection: Arc<GenerateTokensStoppersCollection>,
    name: Option<String>,
    reconciliation_queue: Arc<ReconciliationQueue>,
    slot_aggregated_metrics: Arc<SlotAggregatedMetrics>,
    socket_url: String,
}

impl ManagementSocketClientService {
    pub fn new(
        generate_tokens_channel_tx: mpsc::Sender<GenerateTokensChannel>,
        management_addr: SocketAddr,
        name: Option<String>,
        reconciliation_queue: Arc<ReconciliationQueue>,
        slot_aggregated_metrics: Arc<SlotAggregatedMetrics>,
    ) -> Result<Self> {
        let agent_id = Uuid::new_v4();

        Ok(ManagementSocketClientService {
            generate_tokens_channel_tx,
            generate_tokens_stoppers_collection: Arc::new(GenerateTokensStoppersCollection::new()),
            name,
            reconciliation_queue,
            slot_aggregated_metrics,
            socket_url: format!("ws://{management_addr}/api/v1/agent_socket/{agent_id}"),
        })
    }

    async fn handle_incoming_message(
        generate_tokens_channel_tx: mpsc::Sender<GenerateTokensChannel>,
        generate_tokens_stoppers_collection: Arc<GenerateTokensStoppersCollection>,
        message_tx: mpsc::Sender<ManagementJsonRpcMessage>,
        msg: Message,
        pong_tx: mpsc::Sender<Bytes>,
        reconciliation_queue: Arc<ReconciliationQueue>,
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

                    Ok(())
                }
                JsonRpcMessage::Notification(JsonRpcNotification::SetState(SetStateParams {
                    desired_state,
                })) => {
                    reconciliation_queue
                        .register_change_request(desired_state)
                        .await
                }
                JsonRpcMessage::Notification(JsonRpcNotification::StopRequest(request_id)) => {
                    match generate_tokens_stoppers_collection.stop(request_id) {
                        GenerateTokensStopResult::Stopped => Ok(()),
                        GenerateTokensStopResult::RequestNotFound(request_id) => {
                            warn!("Request with ID {request_id} not found for stopping");

                            Ok(())
                        }
                    }
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
                    request:
                        JsonRpcRequest::GenerateTokens(GenerateTokensParams {
                            max_tokens,
                            prompt,
                        }),
                }) => {
                    for _ in 1..10 {
                        println!("1");
                    }

                    let (chunk_sender, mut chunk_receiver) = mpsc::channel::<ChunkResponse>(100);

                    let generate_tokens_channel_tx = generate_tokens_channel_tx.clone();
                    let id_clone = id.clone();
                    let should_stop = generate_tokens_stoppers_collection.register_for(id_clone);

                    generate_tokens_channel_tx
                        .send(GenerateTokensChannel {
                            chunk_sender,
                            params: GenerateTokensParams {
                                max_tokens,
                                prompt,
                            },
                            should_stop: should_stop.clone(),
                        })
                        .await?;

                    for _ in 1..10 {
                        println!("2");
                    }

                    let generate_tokens_stoppers_collection_clone =
                        generate_tokens_stoppers_collection.clone();
                    let should_stop_clone = should_stop.clone();

                    rt::spawn(async move {
                        for _ in 1..10 {
                            println!("3");
                        }

                        while let Some(chunk) = chunk_receiver.recv().await {
                            if should_stop_clone.load(Relaxed) {
                                break;
                            }

                            for _ in 1..10 {
                                println!("4");
                            }

                            match chunk {
                                ChunkResponse::Data(generated_token) => {
                                    for _ in 1..10 {
                                        println!("5");
                                    }
                                    message_tx
                                        .send(ManagementJsonRpcMessage::Response(
                                            ResponseEnvelope::StreamChunk {
                                                request_id: id.clone(),
                                                chunk: JsonRpcResponse::GeneratedToken(
                                                    generated_token,
                                                ),
                                            },
                                        ))
                                        .await
                                        .unwrap_or_else(|err| {
                                            error!("Failed to send chunk response: {err}");
                                        })
                                }
                                ChunkResponse::Error(err) => {
                                    for _ in 1..10 {
                                        println!("6");
                                    }
                                    let msg = format!("Error generating token: {err}");

                                    message_tx
                                        .send(ManagementJsonRpcMessage::Response(
                                            ResponseEnvelope::Error {
                                                request_id: id.clone(),
                                                error: msg.clone(),
                                            },
                                        ))
                                        .await
                                        .unwrap_or_else(|send_err| {
                                            error!("Failed to send error response: {send_err}");
                                        });

                                    error!("{msg}");
                                }
                            }
                        }

                        for _ in 1..10 {
                            println!("7");
                        }

                        generate_tokens_stoppers_collection_clone.clear(id.clone());
                        message_tx
                            .send(ManagementJsonRpcMessage::Response(
                                ResponseEnvelope::StreamDone {
                                    request_id: id,
                                },
                            ))
                            .await
                            .unwrap_or_else(|err| {
                                error!("Failed to send stream done response: {err}");
                            });

                        for _ in 1..10 {
                            println!("8");
                        }
                    });

                    for _ in 1..10 {
                        println!("9");
                    }

                    Ok(())
                }
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
            Message::Ping(payload) => Ok(pong_tx.send(payload).await?),
            Message::Pong(_) => {
                // Pong received, no action needed
                Ok(())
            }
        }
    }

    async fn keep_connection_alive(&self, mut shutdown: broadcast::Receiver<()>) -> Result<()> {
        let (ws_stream, _response) = connect_async(self.socket_url.clone()).await?;

        info!("Connected to management server");

        let (message_tx, mut message_rx) = mpsc::channel::<ManagementJsonRpcMessage>(100);
        let (pong_tx, mut pong_rx) = mpsc::channel::<Bytes>(100);
        let (mut write, mut read) = ws_stream.split();

        let mut shutdown_resubscribed = shutdown.resubscribe();

        rt::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_resubscribed.recv() => {
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

        let generate_tokens_channel_tx_clone = self.generate_tokens_channel_tx.clone();
        let generate_tokens_stoppers_collection_clone =
            self.generate_tokens_stoppers_collection.clone();
        let name_clone = self.name.clone();
        let reconciliation_queue_clone = self.reconciliation_queue.clone();
        let slot_aggregated_metrics_clone = self.slot_aggregated_metrics.clone();
        let slot_aggregated_metrics_slots_total = self.slot_aggregated_metrics.slots_total;

        rt::spawn(async move {
            message_tx
                .send(ManagementJsonRpcMessage::Notification(
                    ManagementJsonRpcNotification::RegisterAgent(RegisterAgentParams {
                        name: name_clone,
                        slots_total: slot_aggregated_metrics_slots_total,
                    }),
                ))
                .await
                .unwrap_or_else(|err| {
                    error!("Failed to send register agent notification: {err}");
                });

            loop {
                tokio::select! {
                    _ = shutdown.recv() => {
                        info!("Shutdown signal received, closing connection");
                        generate_tokens_stoppers_collection_clone.stop_all();
                        message_tx.send(
                            ManagementJsonRpcMessage::Notification(
                                ManagementJsonRpcNotification::DeregisterAgent,
                            )
                        ).await.unwrap_or_else(|err| {
                            error!("Failed to send deregister agent notification: {err}");
                        });

                        break;
                    }
                    _ = slot_aggregated_metrics_clone.update_notifier.notified() => {
                        // writer.send_rpc_message(ManagementJsonRpcNotification::UpdateSlots {
                        //     slots_processing: self.slot_aggregated_metrics.slots_processing.get(),
                        // }).await?;
                    }
                    msg = read.next() => {
                        match msg {
                            Some(Ok(msg)) => {
                                if let Err(err) = Self::handle_incoming_message(
                                        generate_tokens_channel_tx_clone.clone(),
                                        generate_tokens_stoppers_collection_clone.clone(),
                                        message_tx.clone(),
                                        msg,
                                        pong_tx.clone(),
                                        reconciliation_queue_clone.clone(),
                                    )
                                    .await
                                    .context("Failed to handle incoming message")
                                {
                                    error!("Error handling incoming message: {err}");
                                    break;
                                }
                            }
                            Some(Err(err)) => {
                                error!("Error reading message: {err}");
                                break;
                            }
                            None => break,
                        }
                    }
                }
            }
        })
        .await?;

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
