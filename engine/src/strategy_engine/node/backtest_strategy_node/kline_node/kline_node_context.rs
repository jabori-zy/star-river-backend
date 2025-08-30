use types::cache::key::KlineKey;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use types::strategy::node_event::{KlineSeriesMessage, BacktestNodeEvent};
use types::strategy::node_event::backtest_node_event::kline_node_event::{KlineNodeEvent, KlineUpdateEvent};
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
use types::cache::Key;
use event_center::CommandPublisher;
use event_center::response::cache_engine_response::CacheEngineResponse;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use tokio::sync::oneshot;
use tracing::instrument;
use super::kline_node_type::KlineNodeBacktestConfig;
use types::strategy::node_event::SignalEvent;
use event_center::strategy_event::{StrategyEvent,BacktestStrategyData};
use types::cache::CacheValue;
use event_center::strategy_event::backtest_strategy_event::BacktestStrategyEvent;
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::custom_type::PlayIndex;

#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub exchange_is_registered: Arc<RwLock<bool>>,
    pub data_is_loaded: Arc<RwLock<bool>>,
    pub backtest_config: KlineNodeBacktestConfig,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
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
        let node_id = self.base_context.node_id.clone();
        self.base_context.output_handles.get(&format!("{}_default_output", node_id)).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let _event = event;
        Ok(())
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), String> {
        // tracing::info!("{}: 收到消息: {:?}", self.base_context.node_id, node_event);
        // 收到消息之后，获取对应index的k线数据
        
        match node_event {
            BacktestNodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::KlinePlay(play_event) => {
                        // 提前获取配置信息，统一错误处理
                        let exchange_config = self.backtest_config.exchange_mode_config.as_ref()
                            .ok_or("Exchange mode config is not set")?;
                        
                        // let current_play_index = self.get_play_index().await;
                        // tracing::debug!("current_play_index: {}", current_play_index);

                        let current_play_index = self.get_play_index();
                        tracing::debug!("{}: 接受到k线播放信号。信号的play_index: {}，节点的play_index: {}", self.base_context.node_id, play_event.play_index, current_play_index);
                        
                        // 如果索引不匹配，提前返回错误日志
                        if current_play_index != play_event.play_index {
                            tracing::error!(
                                node_id = %self.base_context.node_id, 
                                node_name = %self.base_context.node_name, 
                                kline_cache_index = %play_event.play_index,
                                signal_index = %current_play_index, 
                                "kline cache index is not equal to signal index"
                            );
                            return Ok(());
                        }

                        // 提取公共数据

                        let exchange = exchange_config.selected_account.exchange.clone();
                        let start_time = exchange_config.time_range.start_date.to_string();
                        let end_time = exchange_config.time_range.end_date.to_string();
                        
                        // 循环处理所有选定的交易对
                        for symbol_config in exchange_config.selected_symbols.iter() {
                            // 创建k线缓存键
                            let backtest_kline_key = KlineKey::new(
                                exchange.clone(),
                                symbol_config.symbol.clone(),
                                symbol_config.interval.clone(),
                                Some(start_time.clone()),
                                Some(end_time.clone()),
                            );
                            
                            // 获取k线缓存值
                            let kline_cache_value = match self.get_history_kline_cache(&backtest_kline_key, current_play_index).await {
                                Ok(value) => value,
                                Err(e) => {
                                    tracing::error!(
                                        node_id = %self.base_context.node_id, 
                                        node_name = %self.base_context.node_name,
                                        symbol = %symbol_config.symbol,
                                        "Failed to get history kline cache: {}", e
                                    );
                                    continue;
                                }
                            };

                            // 发送K线更新事件的通用函数
                            
                            let send_kline_event = |handle_id: String, output_handle: NodeOutputHandle| {
                                let kline_update_event = self.get_kline_update_event(
                                    handle_id,
                                    symbol_config.config_id,
                                    &backtest_kline_key,
                                    current_play_index,
                                    kline_cache_value.clone(),
                                );
                                // tracing::debug!("send_kline_event: {:?}", kline_update_event);
                                let kline_node_event = BacktestNodeEvent::KlineNode(kline_update_event);

                                let _ = output_handle.send(kline_node_event);
                            };

                            // 发送到交易对特定的输出handle
                            let symbol_handle_id = symbol_config.output_handle_id.clone();
                            if let Some(symbol_output_handle) = self.base_context.output_handles.get(&symbol_handle_id) {
                                send_kline_event(symbol_handle_id, symbol_output_handle.clone());
                            }

                            // 发送到默认输出handle
                            let default_output_handle = self.get_default_output_handle();
                            send_kline_event(default_output_handle.output_handle_id.clone(), default_output_handle);

                            // 发送到策略输出handle
                            let strategy_output_handle = self.get_strategy_output_handle();
                            send_kline_event(strategy_output_handle.output_handle_id.clone(), strategy_output_handle.clone());
                        }
                    }
                    _ => {}
                }
            }
            _ => {}


        }



        Ok(())
    }

    // 处理策略内部事件
    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            // StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
            //     // 更新k线缓存索引
            //     self.set_play_index(play_index_update_event.play_index).await;
            //     let strategy_output_handle = self.get_strategy_output_handle();
            //     let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
            //         from_node_id: self.get_node_id().clone(),
            //         from_node_name: self.get_node_name().clone(),
            //         from_node_handle_id: strategy_output_handle.output_handle_id.clone(),
            //         play_index: self.get_play_index().await,
            //         message_timestamp: get_utc8_timestamp_millis(),
            //     }));
            //     if let Err(e) = strategy_output_handle.send(signal) {
            //         tracing::error!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "send event failed: {}", e);
            //     }
                
            // }
            StrategyInnerEvent::NodeReset => {
                // tracing::info!("{}: 收到节点重置事件", self.base_context.node_id);
            }
        }
        Ok(())
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) -> Result<(), String> {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
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
            account_id: self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.account_id.clone(),
            exchange: self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone(),
            sender: self.base_context.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let register_exchange_command = ExchangeEngineCommand::RegisterExchange(register_param);
        self.get_command_publisher().send(register_exchange_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        Ok(response)
    }

    // 从交易所获取k线历史
    #[instrument(skip(self))]
    pub async fn load_kline_history_from_exchange(&mut self) -> Result<bool, String> {
        tracing::info!(node_id = %self.base_context.node_id, node_name = %self.base_context.node_name, "start to load backtest kline data from exchange");
        // 已配置的symbol
        let selected_symbols = self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_symbols.clone();

        let mut is_all_success = true;
        // 遍历每一个symbol，从交易所获取k线历史
        for symbol in selected_symbols.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let params = GetKlineHistoryParams {
                strategy_id: self.base_context.strategy_id.clone(),
                node_id: self.base_context.node_id.clone(),
                account_id: self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.account_id.clone(),
                exchange: self.backtest_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone(),
                symbol: symbol.symbol.clone(),
                interval: symbol.interval.clone(),
                time_range: self.backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone(),
                sender: self.base_context.node_id.clone(),
                timestamp: get_utc8_timestamp_millis(),
                responder: resp_tx,
            };
            let get_kline_history_command = MarketEngineCommand::GetKlineHistory(params);
            self.get_command_publisher().send(get_kline_history_command.into()).await.unwrap();
            let response = resp_rx.await.unwrap();
            if !response.success() {
                is_all_success = false;
                break;
            }

        }
        Ok(is_all_success)
    }

    // 从缓存引擎获取k线数据
    pub async fn get_history_kline_cache(&self,
                                         kline_key: &KlineKey,
                                         play_index: i32, // 缓存索引
    ) -> Result<Vec<Arc<CacheValue>>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: self.get_strategy_id().clone(),
            node_id: self.get_node_id().clone(),
            key: kline_key.clone().into(),
            index: Some(play_index as u32),
            limit: Some(1),
            sender: self.get_node_id().clone(),
            timestamp: get_utc8_timestamp_millis(),
            responder: resp_tx,
        };

        let get_cache_command = CacheEngineCommand::GetCache(params);
        self.get_command_publisher().send(get_cache_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
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

    fn get_kline_update_event(
        &self,
        handle_id: String,
        config_id: i32,
        kline_key: &KlineKey,
        index: i32, // 缓存索引
        kline_data: Vec<Arc<CacheValue>>,
    ) -> KlineNodeEvent {

        KlineNodeEvent::KlineUpdate(KlineUpdateEvent {
            from_node_id: self.get_node_id().clone(),
            from_node_name: self.get_node_name().clone(),
            from_handle_id: handle_id,
            config_id: config_id,
            play_index: index,
            kline_key: kline_key.clone(),
            kline: kline_data,
            timestamp: get_utc8_timestamp_millis(),
        })
    }

}