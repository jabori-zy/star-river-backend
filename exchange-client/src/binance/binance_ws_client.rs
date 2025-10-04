// #![allow(dead_code, unused_imports)]
use crate::binance::url::BinanceWsUrl;
use crate::binance::websocket::Stream;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error, Message, handshake::client::Response},
};

pub struct BinanceWsClient;

impl BinanceWsClient {
    pub async fn connect(url: &str) -> Result<(WebSocketState, Response), Error> {
        let start = std::time::Instant::now();
        let (socket, response) = connect_async(url).await?;
        let duration = start.elapsed();
        tracing::info!(
            "连接至币安websocket服务器成功, 耗时: {:?}。 响应状态: {:?}",
            duration,
            response.status()
        );

        Ok((WebSocketState::new(socket), response))
    }

    pub async fn connect_default() -> Result<(WebSocketState, Response), Error> {
        BinanceWsClient::connect(BinanceWsUrl::BaseUrl.to_string().as_str()).await
    }
}

#[derive(Debug)]
pub struct WebSocketState {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    id: u64,
}

impl WebSocketState {
    pub fn new(socket: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self { socket, id: 0 }
    }

    async fn send<'a, I>(&mut self, method: &str, params: I) -> u64
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut params_str = params.into_iter().map(|s| format!("\"{}\"", s)).collect::<Vec<String>>().join(",");

        if !params_str.is_empty() {
            params_str = format!("\"params\": [{params}],", params = params_str)
        }

        let id = self.id.clone();
        self.id += 1;

        let s = format!(
            "{{\"method\":\"{method}\",
            {params}\"id\":{id}}}",
            method = method,
            params = params_str,
            id = id
        );

        let message = Message::text(s);

        // tracing::debug!("Sent message: {:?}", message);

        self.socket.send(message).await.expect("发送消息失败");

        id
    }

    pub async fn subscribe(&mut self, streams: impl IntoIterator<Item = &Stream>) -> u64 {
        self.send("SUBSCRIBE", streams.into_iter().map(|s| s.as_str())).await
    }

    pub async fn unsubscribe(&mut self, streams: impl IntoIterator<Item = &Stream>) -> u64 {
        self.send("UNSUBSCRIBE", streams.into_iter().map(|s| s.as_str())).await
    }

    pub async fn subscribe_list(&mut self) -> u64 {
        self.send("LIST_SUBSCRIPTIONS", vec![]).await
    }
}

impl AsMut<WebSocketStream<MaybeTlsStream<TcpStream>>> for WebSocketState {
    fn as_mut(&mut self) -> &mut WebSocketStream<MaybeTlsStream<TcpStream>> {
        &mut self.socket
    }
}
