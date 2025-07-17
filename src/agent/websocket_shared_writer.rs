use anyhow::Result;
use async_trait::async_trait;
use futures::stream::SplitSink;
use futures_util::SinkExt as _;
use serde::Serialize;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

use crate::sends_rpc_message::SendsRpcMessage;

pub struct WebSocketSharedWriter {
    writer_mutex: Mutex<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
}

impl WebSocketSharedWriter {
    pub fn new(writer: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) -> Self {
        WebSocketSharedWriter {
            writer_mutex: Mutex::new(writer),
        }
    }

    pub async fn send(&self, message: Message) -> Result<()> {
        let mut writer = self.writer_mutex.lock().await;

        Ok(writer.send(message).await?)
    }
}

#[async_trait]
impl SendsRpcMessage for WebSocketSharedWriter {
    async fn send_rpc_message<TMessage: Send + Serialize>(&self, message: TMessage) -> Result<()> {
        let serialized_message = serde_json::to_string(&message)?;
        let message = Message::Text(serialized_message.into());

        self.send(message).await
    }
}
