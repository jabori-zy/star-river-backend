use super::{
    MetaTrader5,
    ExchangeStreamExt,
    ExchangeClientError,
    Mt5WsClient,
    Mt5KlineInterval,
    WebSocketSnafu,
    KlineInterval,

};
use async_trait::async_trait;
use snafu::ResultExt;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use futures::StreamExt;
use futures::SinkExt;




#[async_trait]
impl ExchangeStreamExt for MetaTrader5 {
    async fn connect_websocket(&mut self) -> Result<(), ExchangeClientError> {
        let (websocket_state, _) = Mt5WsClient::connect_default(self.server_port).await.context(WebSocketSnafu {
            message: "connect to metatrader5 websocket server failed".to_string(),
            account_id: self.terminal_id,
            url: format!("ws://localhost:{}/ws", self.server_port),
        })?;
        self.websocket_state.lock().await.replace(websocket_state);
        Ok(())
    }

    async fn subscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError> {
        let mt5_interval = Mt5KlineInterval::from(interval).to_string();
        let mut mt5_ws_client = self.websocket_state.lock().await;
        tracing::debug!("Metatrader5订阅k线流: {:?}, {:?}, {:?}", symbol, mt5_interval, frequency);
        if let Some(state) = mt5_ws_client.as_mut() {
            let params = json!({
                "symbol": symbol,
                "interval": mt5_interval,
            });
            tracing::debug!("Metatrader5订阅k线流参数: {:?}", params);

            state
                .subscribe(Some("kline"), Some(params), Some(frequency))
                .await
                .expect("订阅k线流失败");
        }
        Ok(())
    }

    async fn unsubscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), ExchangeClientError> {
        tracing::info!("取消订阅k线流: {:?}", symbol);
        let mt5_interval = Mt5KlineInterval::from(interval).to_string();
        let mut mt5_ws_client = self.websocket_state.lock().await;
        if let Some(state) = mt5_ws_client.as_mut() {
            let params = json!({
                "symbol": symbol,
                "interval": mt5_interval,
            });

            state
                .unsubscribe(Some("kline"), Some(params), Some(frequency))
                .await
                .expect("取消订阅k线流失败");
        }
        Ok(())
    }

    async fn get_socket_stream(&self) -> Result<(), ExchangeClientError> {
        // 判断当前是否正在处理流
        if self.is_process_stream.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!("metatrader5已开始处理流数据, 无需重复获取!");
            return Ok(());
        }
        tracing::debug!("metatrader5开始处理流数据");
        // 如果当前没有处理流，则开始处理流,设置状态为true
        self.is_process_stream.store(true, std::sync::atomic::Ordering::Relaxed);

        let websocket_state = self.websocket_state.clone();
        let data_processor = self.data_processor.clone();

        let future = async move {
            loop {
                let receive_message = {
                    let mut websocket_state = websocket_state.lock().await;
                    if let Some(state) = websocket_state.as_mut() {
                        state.as_mut().next().await
                    } else {
                        None
                    }
                }; // 锁在这里被释放

                // 处理原始数据
                if let Some(Ok(msg)) = receive_message {
                    match msg {
                        Message::Ping(data) => {
                            // tracing::debug!("收到ping帧");
                            let mut websocket_state = websocket_state.lock().await;
                            if let Some(state) = websocket_state.as_mut() {
                                // 回复pong帧
                                let socket = state.as_mut();
                                socket.send(Message::Pong(data)).await.expect("发送pong帧失败");
                                // tracing::debug!("发送pong帧");
                            }
                        }
                        Message::Pong(_) => {
                            tracing::debug!("收到pong帧");
                        }
                        Message::Text(text) => {
                            let stream_json =
                                serde_json::from_str::<serde_json::Value>(&text.to_string()).expect("解析WebSocket消息JSON失败");
                            // tracing::debug!("收到消息: {:?}", stream_json);
                            let data_processor = data_processor.lock().await;
                            if let Err(e) = data_processor.process_stream(stream_json).await {
                                tracing::error!("Failed to process stream data: {}", e);
                                // Consider reconnection logic
                            }
                        }
                        _ => {
                            tracing::debug!("收到其他类型的消息: {:?}", msg);
                        }
                    }
                }
            }
        };
        tokio::spawn(future);
        Ok(())
    }
}