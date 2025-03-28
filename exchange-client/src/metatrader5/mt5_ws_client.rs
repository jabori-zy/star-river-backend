use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error, Message, handshake::client::Response},
    MaybeTlsStream, WebSocketStream,
};

use serde::Serialize;
use serde_json::json;
use crate::metatrader5::url::Mt5WsUrl;

pub struct Mt5WsClient;


impl Mt5WsClient {
    pub async fn connect(url: &str) -> Result<(WebSocketState, Response), Error> {
        let start = std::time::Instant::now();
        let (socket, response) = connect_async(url).await?;
        let duration = start.elapsed();
        tracing::info!("连接至metatrader5 websocket服务器成功, 耗时: {:?}。 响应状态: {:?}", duration, response.status());
        
        Ok((WebSocketState::new(socket), response))
    }

    pub async fn connect_default() -> Result<(WebSocketState, Response), Error> {
        Mt5WsClient::connect(Mt5WsUrl::BaseUrl.to_string().as_str()).await
    }
}



pub struct WebSocketState {
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>
}

impl WebSocketState {
    pub fn new(socket: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            socket,
        }
    }

    async fn send<'a, I>(&mut self, command: &str, data_type: Option<&str>, params: Option<I>, frequency: Option<u32>)
    where
        I: Serialize + 'a,
    {
        // 构建消息
        let mut message = json!({
            "command": command
        });
        
        // 添加data_type（如果有）
        if let Some(dt) = data_type {
            message["data_type"] = json!(dt);
        }
        
        // 添加params（如果有）
        if let Some(p) = params {
            // 直接序列化params对象
            if let Ok(params_value) = serde_json::to_value(p) {
                message["params"] = params_value;
            } else {
                eprintln!("无法序列化params");
                return;
            }
        }
        
        // 添加frequency（如果有）
        if let Some(freq) = frequency {
            message["frequency"] = json!(freq);
        }

        let message = Message::text(message.to_string());
        // 发送消息
        self.socket.send(message).await.expect("发送消息失败");
    }

    pub async fn subscribe<I>(&mut self, data_type: Option<&str>, params: Option<I>, frequency: Option<u32>) -> Result<(), Box<dyn std::error::Error>> 
    where
        I: Serialize + 'static
    {
        self.send("subscribe", data_type, params, frequency).await;
        Ok(())
    }

    pub async fn unsubscribe<I>(&mut self, data_type: Option<&str>, params: Option<I>, frequency: Option<u32>) -> Result<(), Box<dyn std::error::Error>> 
    where
        I: Serialize + 'static
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


