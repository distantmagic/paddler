use std::sync::Arc;

use actix_web::rt;
use actix_web::web::Payload;
use actix_web::Error;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_ws::AggregatedMessage;
use actix_ws::Session;
use anyhow::Context as _;
use anyhow::Result;
use async_trait::async_trait;
use futures_util::StreamExt as _;
use log::debug;
use log::error;
use serde::de::DeserializeOwned;
use tokio::sync::broadcast;
use tokio::time::interval;
use tokio::time::Duration;
use tokio::time::MissedTickBehavior;

const MAX_CONTINUATION_SIZE: usize = 100 * 1024;
const PING_INTERVAL: Duration = Duration::from_secs(3);

pub enum ContinuationDecision {
    Continue,
    Stop,
}

#[async_trait]
pub trait ControlsWebSocketEndpoint: Send + Sync + 'static {
    type Context: Send + Sync + 'static;
    type Message: DeserializeOwned + Send + Sync + 'static;

    fn create_context(&self) -> Self::Context;

    async fn handle_deserialized_message(
        context: Arc<Self::Context>,
        deserialized_message: Self::Message,
        mut session: Session,
        shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision>;

    async fn handle_aggregated_message(
        context: Arc<Self::Context>,
        msg: Option<Result<AggregatedMessage, actix_ws::ProtocolError>>,
        session: &mut Session,
        shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        match msg {
            Some(Ok(AggregatedMessage::Binary(_))) => {
                debug!("Received binary message, but only text messages are supported");

                Ok(ContinuationDecision::Continue)
            }
            Some(Ok(AggregatedMessage::Close(_))) => return Ok(ContinuationDecision::Stop),
            Some(Ok(AggregatedMessage::Ping(msg))) => {
                if session.pong(&msg).await.is_err() {
                    return Ok(ContinuationDecision::Stop);
                }

                Ok(ContinuationDecision::Continue)
            }
            Some(Ok(AggregatedMessage::Pong(_))) => {
                // ignore pong messages
                Ok(ContinuationDecision::Continue)
            }
            Some(Ok(AggregatedMessage::Text(text))) => {
                match Self::handle_text_message(
                    context.clone(),
                    session.clone(),
                    shutdown_tx.clone(),
                    &text,
                )
                .await
                .context(format!("Text message: {text}"))
                {
                    Ok(continuation_decision) => return Ok(continuation_decision),
                    Err(err) => {
                        error!("Error handling message: {err:?}");

                        Ok(ContinuationDecision::Continue)
                    }
                }
            }
            Some(Err(err)) => {
                error!("Error receiving message: {err:?}");

                return Ok(ContinuationDecision::Stop);
            }
            None => return Ok(ContinuationDecision::Stop),
        }
    }

    async fn handle_serialization_error(
        _context: Arc<Self::Context>,
        error: serde_json::Error,
        _session: Session,
        _shutdown_tx: broadcast::Sender<()>,
    ) -> Result<ContinuationDecision> {
        error!("Paddler-RPC serializatikon error: {error}");

        Ok(ContinuationDecision::Continue)
    }

    async fn handle_text_message(
        context: Arc<Self::Context>,
        session: Session,
        shutdown_tx: broadcast::Sender<()>,
        text: &str,
    ) -> Result<ContinuationDecision> {
        match serde_json::from_str::<Self::Message>(text) {
            Ok(deserialized_message) => {
                Self::handle_deserialized_message(
                    context,
                    deserialized_message,
                    session,
                    shutdown_tx,
                )
                .await
            }
            Err(
                err @ serde_json::Error {
                    ..
                },
            ) if err.is_data() || err.is_syntax() => {
                error!("JSON-RPC syntax error: {err:?}");

                Self::handle_serialization_error(context, err, session, shutdown_tx).await
            }
            Err(err) => {
                error!("Error handling JSON-RPC request: {err:?}");

                Self::handle_serialization_error(context, err, session, shutdown_tx).await
            }
        }
    }

    async fn on_connection_start(
        _context: Arc<Self::Context>,
        _session: &mut Session,
    ) -> Result<ContinuationDecision> {
        Ok(ContinuationDecision::Continue)
    }

    fn respond(&self, payload: Payload, req: HttpRequest) -> Result<HttpResponse, Error> {
        let context = Arc::new(self.create_context());
        let (res, mut session, msg_stream) = actix_ws::handle(&req, payload)?;
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(2);

        let mut aggregated_msg_stream = msg_stream
            .aggregate_continuations()
            .max_continuation_size(MAX_CONTINUATION_SIZE);

        rt::spawn(async move {
            match Self::on_connection_start(context.clone(), &mut session).await {
                Ok(ContinuationDecision::Continue) => {}
                Ok(ContinuationDecision::Stop) => return,
                Err(err) => {
                    error!("Error in connection start handler: {err:?}");

                    return;
                }
            }

            let mut ping_ticker = interval(PING_INTERVAL);

            ping_ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

            loop {
                tokio::select! {
                    msg = aggregated_msg_stream.next() => {
                        match Self::handle_aggregated_message(
                            context.clone(),
                            msg,
                            &mut session,
                            shutdown_tx.clone(),
                        ).await {
                            Ok(ContinuationDecision::Continue) => {
                                // continue processing messages
                            }
                            Ok(ContinuationDecision::Stop) => break,
                            Err(err) => error!("Error handling message: {err:?}"),
                        }
                    }
                    _ = ping_ticker.tick() => {
                        if session.ping(b"").await.is_err() {
                            break;
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        break;
                    }
                }
            }

            let _ = session.close(None).await;
        });

        Ok(res)
    }
}
