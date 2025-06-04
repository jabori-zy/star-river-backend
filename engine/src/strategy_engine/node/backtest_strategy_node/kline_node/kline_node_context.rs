
use types::cache::cache_key::BacktestKlineCacheKey;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use types::strategy::node_event::{KlineSeriesMessage, NodeEvent, BacktestKlineUpdateEvent};
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use event_center::response::Response;
use crate::strategy_engine::node::node_context::{BacktestNodeContextTrait,BacktestBaseNodeContext};
use event_center::command::market_engine_command::{MarketEngineCommand, GetKlineHistoryParams};
use event_center::command::exchange_engine_command::RegisterExchangeParams;
use event_center::command::exchange_engine_command::ExchangeEngineCommand;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::Mutex;
use heartbeat::Heartbeat;
use event_center::command::cache_engine_command::{CacheEngineCommand, GetCacheParams};
use types::cache::CacheKey;
use event_center::CommandPublisher;
use event_center::response::cache_engine_response::CacheEngineResponse;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use tokio::sync::oneshot;
use tracing::instrument;
use super::kline_node_type::KlineNodeBacktestConfig;
use types::strategy::node_event::SignalEvent;
use event_center::strategy_event::{StrategyEvent,BacktestStrategyData};
use types::cache::CacheValue;


#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub data_is_loaded: Arc<RwLock<bool>>,
    pub backtest_config: KlineNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    pub kline_cache_index: Arc<RwLock<u32>>,
}

#[async_trait]
impl BacktestNodeContextTrait for KlineNodeContext {

    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("kline_node_output")).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let _event = event;
        Ok(())
    }

    async fn handle_node_event(&mut self, message: NodeEvent) -> Result<(), String> {
        // tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, message);
        // 收到消息之后，获取对应index的k线数据
        let exchange = self.backtest_config.exchange_config.as_ref().unwrap().selected_data_source.exchange.clone();
        let symbol = self.backtest_config.exchange_config.as_ref().unwrap().symbol.clone();
        let interval = self.backtest_config.exchange_config.as_ref().unwrap().interval.clone();
        let start_time = self.backtest_config.exchange_config.as_ref().unwrap().time_range.start_date.to_string();
        let end_time = self.backtest_config.exchange_config.as_ref().unwrap().time_range.end_date.to_string();
        let backtest_kline_cache_key= BacktestKlineCacheKey::new(exchange, symbol, interval, start_time, end_time);
        
        match message {
            NodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::KlineTick(kline_tick_event) => {

                        // 如果k线缓存索引与信号索引相同，则发送回测数据更新事件
                        if *self.kline_cache_index.read().await == kline_tick_event.signal_index {
                            let cache_key: CacheKey = backtest_kline_cache_key.clone().into();
                            let kline_cache_value = self.get_history_kline_cache(cache_key.clone(), kline_tick_event.signal_index).await.unwrap();
                            // 发送回测数据更新事件
                            let cache_data: Vec<Vec<f64>> = kline_cache_value.into_iter().map(|cache_value| cache_value.to_list()).collect();
                                            
                            let strategy_data = BacktestStrategyData {
                                strategy_id: self.base_context.strategy_id.clone(),
                                cache_key: cache_key.get_key(),
                                data: cache_data[0].clone(),
                                timestamp: get_utc8_timestamp_millis(),
                            };
                            let strategy_event = StrategyEvent::BacktestStrategyDataUpdate(strategy_data);
                            // tracing::info!("{}: 发送回测数据更新事件", self.base_context.strategy_id);
                            self.get_event_publisher().publish(strategy_event.into()).await.unwrap();

                            // 发送回测K线更新事件
                            let kline_message = BacktestKlineUpdateEvent {
                                from_node_id: self.base_context.node_id.clone(),
                                from_node_name: self.base_context.node_name.clone(),
                                from_node_handle_id: self.base_context.node_id.clone(),
                                kline_cache_index: kline_tick_event.signal_index,
                                kline_cache_key: backtest_kline_cache_key.clone(),
                                kline: cache_data[0].clone(),
                                message_timestamp: get_utc8_timestamp_millis(),
                            };
                            let kline_event = NodeEvent::BacktestKline(kline_message);
                            self.get_default_output_handle().send(kline_event).unwrap();

                        } else {
                            tracing::error!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "kline cache index is not equal to signal index");
                        }
                        
                    }
                    _ => {}
                }
            }
            _ => {}


        }



        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
                // 更新k线缓存索引
                *self.kline_cache_index.write().await = play_index_update_event.played_index;
                tracing::debug!("{}: 更新k线缓存索引: {}", self.get_node_id(), play_index_update_event.played_index);
                
            }
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
    pub async fn get_history_kline_cache(&self,
        kline_cache_key: CacheKey,
        index: u32, // 缓存索引
    ) -> Result<Vec<Arc<CacheValue>>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: self.get_strategy_id().clone(),
            node_id: self.get_node_id().clone(),
            cache_key: kline_cache_key.clone(),
            index: Some(index),
            limit: Some(1),
            sender: self.get_node_id().clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let get_cache_command = CacheEngineCommand::GetCache(params);
        self.get_command_publisher().send(get_cache_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.code() == 0 {
            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                match cache_reponse {
                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                        return Ok(get_cache_data_response.cache_data);
                    }
                    _ => {}
                }
            }
        }
        Err(format!("get history kline cache failed"))
    }

}