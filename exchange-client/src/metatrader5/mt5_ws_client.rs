use exchange_core::exchange_trait::WebSocketClient;
use futures::SinkExt;
use serde::Serialize;
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Message, handshake::client::Response},
};
use tracing::instrument;

#[derive(Debug)]
pub struct Mt5WsClient;

impl WebSocketClient for Mt5WsClient {}

impl Mt5WsClient {
    #[instrument]
    pub async fn connect(url: &str) -> Result<(WebSocketState, Response), tokio_tungstenite::tungstenite::error::Error> {
        let start = std::time::Instant::now();
        let (socket, response) = connect_async(url).await?;
        let duration = start.elapsed();
        tracing::info!(
            "connect to metatrader5 websocket server successfully, duration: {:?}, response status: {:?}",
            duration,
            response.status()
        );

        Ok((WebSocketState::new(socket), response))
    }

    pub async fn connect_default(port: u16) -> Result<(WebSocketState, Response), tokio_tungstenite::tungstenite::error::Error> {
        let url = format!("ws://localhost:{}/ws", port);
        tracing::debug!("ws url: {:?}", url);
        Mt5WsClient::connect(&url).await
    }
}

#[derive(Debug)]
pub struct WebSocketState {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebSocketState {
    pub fn new(socket: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self { socket }
    }

    async fn send<'a, I>(&mut self, command: &str, data_type: Option<&str>, params: Option<I>, frequency: Option<u32>)
    where
        I: Serialize + 'a,
    {
        // Build message
        let mut message = json!({
            "command": command
        });

        // Add data_type if provided
        if let Some(dt) = data_type {
            message["data_type"] = json!(dt);
        }

        // Add params if provided
        if let Some(p) = params {
            // Directly serialize params object
            if let Ok(params_value) = serde_json::to_value(p) {
                message["params"] = params_value;
            } else {
                eprintln!("Failed to serialize params");
                return;
            }
        }

        // Add frequency if provided
        if let Some(freq) = frequency {
            message["frequency"] = json!(freq);
        }

        let message = Message::text(message.to_string());
        tracing::debug!("Sending message: {:?}", message);
        // Send message
        self.socket.send(message).await.expect("Failed to send message");
    }

    pub async fn subscribe<I>(
        &mut self,
        data_type: Option<&str>,
        params: Option<I>,
        frequency: Option<u32>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        I: Serialize + 'static,
    {
        self.send("subscribe", data_type, params, frequency).await;
        Ok(())
    }

    pub async fn unsubscribe<I>(
        &mut self,
        data_type: Option<&str>,
        params: Option<I>,
        frequency: Option<u32>,
    ) -> Result<(), Box<dyn std::error::Error>>
    where
        I: Serialize + 'static,
    {
        self.send("unsubscribe", data_type, params, frequency).await;
        Ok(())
    }
}

impl AsMut<WebSocketStream<MaybeTlsStream<TcpStream>>> for WebSocketState {
    fn as_mut(&mut self) -> &mut WebSocketStream<MaybeTlsStream<TcpStream>> {
        &mut self.socket
    }
}
