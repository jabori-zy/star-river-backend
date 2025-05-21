use event_center::command::Command;
use sea_orm::sea_query::Index;
use types::cache::cache_key::HistoryKlineCacheKey;
use types::market::KlineInterval;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use types::strategy::node_message::{KlineSeriesMessage, NodeMessage};
use uuid::Uuid;
use event_center::response::Response;
use crate::strategy_engine::node::node_context::{NodeContextTrait,BaseNodeContext};
use event_center::command::market_engine_command::{MarketEngineCommand, GetKlineHistoryParams};
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
use types::cache::{CacheKey, cache_key::KlineCacheKey};
use event_center::CommandPublisher;
use event_center::response::cache_engine_response::CacheEngineResponse;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use tokio::sync::oneshot;
use event_center::response::ResponseTrait;
use tracing::instrument;
use super::kline_node_type::KlineNodeBacktestConfig;
use types::strategy::node_message::SignalType;




#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BaseNodeContext,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub data_is_loaded: Arc<RwLock<bool>>,
    pub backtest_config: KlineNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
}

#[async_trait]
impl NodeContextTrait for KlineNodeContext {

    fn clone_box(&self) -> Box<dyn NodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("live_data_node_output")).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let _event = event;
        Ok(())
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        // 收到消息之后，直接发送给下一个节点
        let exchange = self.backtest_config.exchange_config.as_ref().unwrap().selected_data_source.exchange.clone();
        let symbol = self.backtest_config.exchange_config.as_ref().unwrap().symbol.clone();
        let interval = self.backtest_config.exchange_config.as_ref().unwrap().interval.clone();
        let start_time = self.backtest_config.exchange_config.as_ref().unwrap().time_range.start_date.to_string();
        let end_time = self.backtest_config.exchange_config.as_ref().unwrap().time_range.end_date.to_string();
        let cache_key = HistoryKlineCacheKey::new(exchange, symbol, interval, start_time, end_time);
        
        match message {
            NodeMessage::Signal(signal_message) => {
                let signal_type = signal_message.signal_type;
                match signal_type {
                    SignalType::FetchKlineData(index) => {
                        let (resp_tx, resp_rx) = oneshot::channel();
                        let get_cache_params = GetCacheParams {
                            strategy_id: self.base_context.strategy_id.clone(),
                            node_id: self.base_context.node_id.clone(),
                            cache_key: cache_key.into(),
                            index: Some(index),
                            limit: Some(1),
                            sender: self.base_context.node_id.clone(),
                            timestamp: get_utc8_timestamp_millis(),
                            responder: resp_tx,
                        };
                        let get_cache_command = CacheEngineCommand::GetCache(get_cache_params);
                        self.get_command_publisher().send(get_cache_command.into()).await.unwrap();

                        // 等待响应
                        let response = resp_rx.await.unwrap();
                        if response.code() == 0 {
                            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                                match cache_reponse {
                                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                                        // tracing::info!("{}: 收到缓存数据: {:?}", self.base_context.node_id, get_cache_data_response.cache_data[0].to_json_with_time());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}


        }



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
            account_id: self.backtest_config.exchange_config.as_ref().unwrap().selected_data_source.account_id.clone(),
            exchange: self.backtest_config.exchange_config.as_ref().unwrap().selected_data_source.exchange.clone(),
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

    // 从交易所获取k线历史
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<Response, String> {
        tracing::info!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "start to load backtest kline data from exchange");
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetKlineHistoryParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            account_id: self.backtest_config.exchange_config.as_ref().unwrap().selected_data_source.account_id.clone(),
            exchange: self.backtest_config.exchange_config.as_ref().unwrap().selected_data_source.exchange.clone(),
            symbol: self.backtest_config.exchange_config.as_ref().unwrap().symbol.clone(),
            interval: self.backtest_config.exchange_config.as_ref().unwrap().interval.clone(),
            time_range: self.backtest_config.exchange_config.as_ref().unwrap().time_range.clone(),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let get_kline_history_command = MarketEngineCommand::GetKlineHistory(params);
        self.get_command_publisher().send(get_kline_history_command.into()).await.unwrap();
        
        // 等待响应
        let response = resp_rx.await.unwrap();
        // tracing::debug!(response = ?response);
        Ok(response)
    }

    // 从缓存引擎获取k线数据
    pub async fn get_history_kline_cache(
        strategy_id: i32, 
        node_id: String,
        node_name: String,
        kline_cache_key: CacheKey,
        index: u32, // 缓存索引
        limit: u32, // 缓存数量(倒着取)
        command_publisher: CommandPublisher,
        output_handle: NodeOutputHandle,
    ){
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: strategy_id,
            node_id: node_id.clone(),
            cache_key: kline_cache_key.clone(),
            index: Some(index),
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
                        output_handle.send(NodeMessage::KlineSeries(kline_series_message)).unwrap();
                    }
                    _ => {}
                }
            }
        }
    }

}