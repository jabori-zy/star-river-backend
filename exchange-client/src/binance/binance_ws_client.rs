// #![allow(dead_code, unused_imports)]
use exchange_core::{error::ExchangeError, exchange_trait::WebSocketClient};
use futures::SinkExt;
use snafu::ResultExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::{Error, Message, handshake::client::Response},
};

use crate::binance::{
    error::{BinanceError, WebSocketConnectionFailedSnafu},
    url::BinanceWsUrl,
    websocket::Stream,
};

#[derive(Debug)]
pub struct BinanceWsBuilder;

impl BinanceWsBuilder {
    pub async fn connect(url: &str) -> Result<(BinanceWebSocket, Response), BinanceError> {
        let start = std::time::Instant::now();
        let (socket, response) = connect_async(url)
            .await
            .context(WebSocketConnectionFailedSnafu { url: url.to_string() })?;
        let duration = start.elapsed();
        tracing::info!(
            "连接至币安websocket服务器成功, 耗时: {:?}。 响应状态: {:?}",
            duration,
            response.status()
        );

        Ok((BinanceWebSocket::new(socket), response))
    }

    pub async fn connect_default() -> Result<(BinanceWebSocket, Response), BinanceError> {
        BinanceWsBuilder::connect(BinanceWsUrl::BaseUrl.to_string().as_str()).await
    }
}

#[derive(Debug)]
pub struct BinanceWebSocket {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
    id: u64,
}

impl WebSocketClient for BinanceWebSocket {}

impl BinanceWebSocket {
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

impl AsMut<WebSocketStream<MaybeTlsStream<TcpStream>>> for BinanceWebSocket {
    fn as_mut(&mut self) -> &mut WebSocketStream<MaybeTlsStream<TcpStream>> {
        &mut self.socket
    }
}
