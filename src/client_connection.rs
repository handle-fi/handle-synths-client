use crate::interface::{MessageId, Request, RequestContent, Response};
use ethers::prelude::StreamExt;
use futures::stream::{SplitSink, SplitStream};
use futures::SinkExt;
use rand::random;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::oneshot::{Receiver, Sender};
use tokio::sync::{oneshot, Mutex};
use tokio_tungstenite::tungstenite::{Error, Message};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

pub type SocketWrite = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type SocketRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Debug, Clone)]
pub struct ClientConnection {
    tx: Arc<Mutex<SocketWrite>>,
    response_listeners: Arc<Mutex<Vec<(MessageId, Sender<Response>)>>>,
}

impl ClientConnection {
    pub async fn connect(ws_url: &str) -> eyre::Result<Self> {
        let (ws_stream, _) = connect_async(ws_url).await?;
        let (tx, rx) = ws_stream.split();
        let response_listeners = Arc::new(Mutex::new(Vec::new()));
        tokio::spawn(listen_for_messages(rx, response_listeners.clone()));
        Ok(Self {
            tx: Arc::new(Mutex::new(tx)),
            response_listeners,
        })
    }

    pub async fn send_raw_message<S>(&self, message: S) -> Result<(), Error>
    where
        S: Into<String>,
    {
        let mut tx = self.tx.lock().await;
        tx.send(Message::text(message)).await
    }

    pub async fn send_request(&self, content: RequestContent) -> eyre::Result<Response> {
        let id = random::<u64>().to_string();
        let request = Request {
            id: Some(id.clone()),
            content,
        };
        let serialized = serde_json::to_string(&request)?;
        let rx = self.add_response_listener(id).await;
        self.send_raw_message(&serialized).await?;
        // TODO: add timeout error handling.
        let response = rx.await?;
        Ok(response)
    }

    async fn add_response_listener(&self, message_id: MessageId) -> Receiver<Response> {
        let mut listeners = self.response_listeners.lock().await;
        let (tx, rx) = oneshot::channel();
        listeners.push((message_id, tx));
        rx
    }
}

async fn listen_for_messages(
    rx: SocketRead,
    response_listeners: Arc<Mutex<Vec<(MessageId, Sender<Response>)>>>,
) {
    rx.for_each(|msg| async {
        let Ok(msg) = msg else {
            return;
        };
        let Ok(text) = msg.to_text() else {
            return;
        };
        let Ok(response) = serde_json::from_str::<Response>(text) else {
            return;
        };
        let Some(response_id) = &response.id else {
            return;
        };
        let mut response_listeners = response_listeners.lock().await;
        let listener_index_opt = response_listeners
            .iter_mut()
            .position(|(id, _)| id == response_id);
        let Some(listener_index) = listener_index_opt else {
            return;
        };
        let (_, sender) = response_listeners.swap_remove(listener_index);
        _ = sender.send(response);
    })
    .await;
}
