use futures::stream::{SplitSink, SplitStream};
use futures::SinkExt;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

pub type SocketWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type SocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Clone)]
pub struct ClientConnection {
    tx: Arc<Mutex<SocketWrite>>,
}

impl ClientConnection {
    pub async fn send_ws_message<S>(&self, message: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let mut tx = self.tx.lock().await;
        tx.send(Message::text(message)).await
    }
}
