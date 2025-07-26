use event_center::command::Command;
use types::market::KlineInterval;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use types::strategy::node_event::{KlineSeriesMessage, BacktestNodeEvent};
use uuid::Uuid;
use event_center::response::Response;
use crate::strategy_engine::node::node_context::{LiveNodeContextTrait,LiveBaseNodeContext};
use event_center::command::market_engine_command::{MarketEngineCommand, SubscribeKlineStreamParams, UnsubscribeKlineStreamParams};
use event_center::command::exchange_engine_command::RegisterExchangeParams;
use event_center::response::market_engine_response::MarketEngineResponse;
use event_center::response::exchange_engine_response::ExchangeEngineResponse;
use event_center::command::exchange_engine_command::ExchangeEngineCommand;
use types::strategy::SelectedAccount;
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use heartbeat::Heartbeat;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use types::cache::{Key, key::KlineKey};
use event_center::CommandPublisher;
use event_center::response::cache_engine_response::CacheEngineResponse;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use tokio::sync::oneshot;
use event_center::response::ResponseTrait;
use tracing::instrument;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineNodeLiveConfig {
    #[serde(rename = "selectedLiveAccount")]
    pub selected_live_account: SelectedAccount,
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_kline_interval")]
    pub interval: KlineInterval,
    // pub frequency: u32,
}

fn deserialize_kline_interval<'de, D>(deserializer: D) -> Result<KlineInterval, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // 将字符串反序列化为String
    let s = String::deserialize(deserializer)?;
    
    // 使用as_str()方法获取&str，然后传递给from_str
    match KlineInterval::from_str(s.as_str()) {
        Ok(interval) => Ok(interval),
        Err(e) => Err(serde::de::Error::custom(format!("无法解析KlineInterval: {}", e)))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataNodeSimulateConfig {
    pub selected_simulate_accounts: SelectedAccount,
    pub symbol: String,
    pub interval: KlineInterval,
    // pub frequency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveDataNodeBacktestConfig {
    pub start_date: String,
    pub end_date: String,
    pub accounts: Vec<i32>,
}



#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: LiveBaseNodeContext,
    pub stream_is_subscribed: Arc<RwLock<bool>>,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub live_config: KlineNodeLiveConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
}

#[async_trait]
impl LiveNodeContextTrait for KlineNodeContext {

    fn clone_box(&self) -> Box<dyn LiveNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &LiveBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut LiveBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("kline_node_output")).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let _event = event;
        Ok(())
    }

    async fn handle_message(&mut self, message: BacktestNodeEvent) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        Ok(())
    }
    
}


impl KlineNodeContext {

    // 注册交易所
    #[instrument(skip(self))]
    pub async fn register_exchange(&mut self) -> Result<Response, String> {
        tracing::info!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "start to register exchange");
        let (resp_tx, resp_rx) = oneshot::channel();
        let register_param = RegisterExchangeParams {
            account_id: self.live_config.selected_live_account.account_id.clone(),
            exchange: self.live_config.selected_live_account.exchange.clone(),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let register_exchange_command = ExchangeEngineCommand::RegisterExchange(register_param);
        self.get_command_publisher().send(register_exchange_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        tracing::debug!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "received register exchange response code: {:?}", response.code());
        Ok(response)
        
        
    }

    #[instrument(skip(self))]
    pub async fn subscribe_kline_stream(&mut self) -> Result<Response, String> {
        tracing::info!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "start to subscribe kline stream");
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = SubscribeKlineStreamParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            account_id: self.live_config.selected_live_account.account_id.clone(),
            exchange: self.live_config.selected_live_account.exchange.clone(),
            symbol: self.live_config.symbol.clone(),
            interval: self.live_config.interval.clone(),
            frequency: 1000,
            cache_size: 20,
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let subscribe_kline_stream_command = MarketEngineCommand::SubscribeKlineStream(params);
        self.get_command_publisher().send(subscribe_kline_stream_command.into()).await.unwrap();
        
        // 等待响应
        let response = resp_rx.await.unwrap();
        tracing::debug!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "received subscribe kline stream response code: {:?}", response.code());
        Ok(response)
    }

    pub async fn unsubscribe_kline_stream(&mut self) -> Result<Response, String> {
        let (account_id, exchange, symbol, interval) = (
            self.live_config.selected_live_account.account_id.clone(), 
            self.live_config.selected_live_account.exchange.clone(), 
            self.live_config.symbol.clone(), 
            self.live_config.interval.clone());
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = UnsubscribeKlineStreamParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            account_id: account_id,
            exchange: exchange,
            symbol: symbol,
            interval: interval,
            frequency: 1000,
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let unsubscribe_kline_stream_command = MarketEngineCommand::UnsubscribeKlineStream(params);
        tracing::debug!("{}取消订阅k线流: {:?}", self.base_context.node_name, unsubscribe_kline_stream_command);
        self.get_command_publisher().send(unsubscribe_kline_stream_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        tracing::info!("{}收到取消订阅k线流响应: {:?}", self.base_context.node_name, response);
        Ok(response)
    }

    pub async fn register_task(&mut self) {
        let kline_cache_key = Key::Kline(KlineKey {
            exchange: self.live_config.selected_live_account.exchange.clone(),
            symbol: self.live_config.symbol.clone(),
            interval: self.live_config.interval.clone(),
            start_time:None,
            end_time:None
        });
        let command_publisher = self.get_command_publisher().clone();
        let strategy_id = self.base_context.strategy_id.clone();
        let node_id = self.base_context.node_id.clone();
        let node_name = self.base_context.node_name.clone();
        let output_handle = self.get_default_output_handle();

        let mut heartbeat = self.heartbeat.lock().await;
        heartbeat.register_async_task(
            format!("{}获取k线数据", node_name),
            move || {
                let kline_cache_key = kline_cache_key.clone();
                let command_publisher = command_publisher.clone();
                let strategy_id = strategy_id.clone();
                let node_id = node_id.clone();
                let node_name = node_name.clone();
                let output_handle = output_handle.clone();
                async move {
                    Self::get_kline_series_cache(
                        strategy_id,
                        node_id,
                        node_name,
                        kline_cache_key,
                        20,
                        command_publisher,
                        output_handle,
                    ).await;
                }
            },
            10
        ).await;
    }

    // 从缓存引擎获取k线数据
    pub async fn get_kline_series_cache(
        strategy_id: i32, 
        node_id: String,
        node_name: String,
        kline_cache_key: Key, 
        limit: u32,
        command_publisher: CommandPublisher,
        output_handle: NodeOutputHandle,
    ){
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: strategy_id,
            node_id: node_id.clone(),
            key: kline_cache_key.clone(),
            index: None,
            limit: Some(limit),
            sender: node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let get_cache_command = CacheEngineCommand::GetCache(params);
        command_publisher.send(get_cache_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                match cache_reponse {
                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                        let kline_series_message = KlineSeriesMessage {
                            from_node_id: node_id.clone(),
                            from_node_name: node_name.clone(),
                            exchange: kline_cache_key.get_exchange(),
                            symbol: kline_cache_key.get_symbol(),
                            interval: kline_cache_key.get_interval(),
                            kline_series: get_cache_data_response.cache_data,
                            message_timestamp: get_utc8_timestamp_millis(),
                        };
                        output_handle.send(BacktestNodeEvent::KlineSeries(kline_series_message)).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

}