use anyhow::Result;
use futures::stream::SplitSink;
use futures_util::SinkExt as _;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::WebSocketStream;

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
