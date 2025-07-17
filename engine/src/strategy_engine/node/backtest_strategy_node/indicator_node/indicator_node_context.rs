use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use async_trait::async_trait;
use event_center::Event;
use event_center::command::Command;
use event_center::response::Response;
use event_center::response::indicator_engine_response::IndicatorEngineResponse;
use event_center::command::indicator_engine_command::{IndicatorEngineCommand, RegisterIndicatorParams};
use utils::get_utc8_timestamp_millis;
use types::strategy::node_event::{IndicatorUpdateEvent, BacktestNodeEvent, IndicatorNodeEvent};
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use event_center::command::cache_engine_command::{AddCacheKeyParams, CacheEngineCommand, GetCacheParams};
use event_center::command::indicator_engine_command::CalculateBacktestIndicatorParams;
use types::cache::key::IndicatorKey;
use event_center::response::cache_engine_response::CacheEngineResponse;
use types::cache::{KeyTrait, CacheValue};
use tokio::sync::oneshot;
use event_center::response::ResponseTrait;
use super::indicator_node_type::IndicatorNodeBacktestConfig;
use types::cache::key::{BacktestIndicatorKey, BacktestKlineKey};
use tokio::time::Duration;
use types::indicator::IndicatorConfig;
use types::indicator::Indicator;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use types::strategy::node_event::{PlayIndexUpdateEvent, SignalEvent};
use types::strategy::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::command::backtest_strategy_command::StrategyCommand;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: IndicatorNodeBacktestConfig,
    pub is_registered: Arc<RwLock<bool>>, // 是否已经注册指标
    pub kline_cache_key: BacktestKlineKey, // 回测K线缓存键
    pub indicator_cache_keys: Vec<BacktestIndicatorKey>, // 指标缓存键
}




#[async_trait]
impl BacktestNodeContextTrait for IndicatorNodeContext {
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
        let default_output_handle_id = format!("{}_default_output", self.base_context.node_id);
        self.base_context.output_handles.get(&default_output_handle_id).unwrap().clone()
    }

    async fn handle_event(&mut self, event: Event) -> Result<(), String> {



        Ok(())
    }

    
    async fn handle_node_event(&mut self, message: BacktestNodeEvent) -> Result<(), String> {
        match message {
            BacktestNodeEvent::KlineNode(kline_event) => {
                // tracing::debug!("{}: 收到回测k线更新事件: {:?}", self.get_node_id(), kline_event);

                // 提前获取配置信息，统一错误处理
                let exchange_config = self.backtest_config.exchange_mode_config.as_ref()
                    .ok_or("Exchange mode config is not set")?;
                
                let current_play_index = self.get_play_index().await;
                
                // 如果索引不匹配，提前返回错误日志
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    if current_play_index != kline_update_event.play_index {
                    tracing::error!(
                        node_id = %self.base_context.node_id, 
                        node_name = %self.base_context.node_name, 
                        kline_cache_index = %kline_update_event.play_index,
                        signal_index = %current_play_index, 
                        "kline cache index is not equal to signal index"
                    );
                    return Ok(());
                }

                // 提取公共数据
                let exchange = exchange_config.selected_account.exchange.clone();
                let selected_symbol = exchange_config.selected_symbol.clone();
                let symbol = selected_symbol.symbol.clone();
                let interval = selected_symbol.interval.clone();
                let time_range = exchange_config.time_range.clone();
                let node_id = self.get_node_id().clone();
                let node_name = self.get_node_name().clone();
                let timestamp = get_utc8_timestamp_millis();
                
                // 遍历指标缓存键，计算指标
                for ind_config in exchange_config.selected_indicators.iter() {
                    let indicator_cache_key = BacktestIndicatorKey { 
                        exchange: exchange.clone(), 
                        symbol: symbol.clone(), 
                        interval: interval.clone(), 
                        indicator_config: ind_config.indicator_config.clone(), 
                        start_time: time_range.start_date.to_string(),
                        end_time: time_range.end_date.to_string() 
                    };

                    let from_handle_id = ind_config.handle_id.clone();
                    
                    // 获取指标缓存数据，增加错误处理
                    let indicator_cache_data = match self.get_backtest_indicator_cache(&indicator_cache_key, kline_update_event.play_index).await {
                        Ok(data) => data,
                        Err(e) => {
                            tracing::error!(
                                node_id = %self.base_context.node_id,
                                node_name = %self.base_context.node_name,
                                indicator = ?ind_config.indicator_config,
                                "Failed to get backtest indicator cache: {}", e
                            );
                            continue;
                        }
                    };

                    // 发送指标更新事件的通用函数
                    let send_indicator_event = |handle_id: String, output_handle: NodeOutputHandle, data: Vec<Arc<CacheValue>>| {
                        let indicator_update_event = IndicatorUpdateEvent {
                            from_node_id: node_id.clone(),
                            from_node_name: node_name.clone(),
                            from_handle_id: handle_id,
                            exchange: indicator_cache_key.get_exchange(),
                            symbol: indicator_cache_key.get_symbol(),
                            interval: indicator_cache_key.get_interval(),
                            indicator_id: ind_config.indicator_id,
                            indicator_config: indicator_cache_key.get_indicator_config(),
                            indicator_key: indicator_cache_key.clone(),
                            indicator_series: data,
                            play_index: kline_update_event.play_index,
                            timestamp: timestamp,
                        };
                        
                        let event = BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event));
                        // tracing::debug!("indicator-node-event: {:?}", serde_json::to_string(&event).unwrap());
                        let _ = output_handle.send(event);
                    };

                    // 发送到指标特定的输出handle
                    if let Some(output_handle) = self.base_context.output_handles.get(&from_handle_id) {
                        send_indicator_event(from_handle_id, output_handle.clone(), indicator_cache_data.clone());
                    }

                    // 发送到默认输出handle
                    let default_output_handle = self.get_default_output_handle();
                    send_indicator_event(default_output_handle.output_handle_id.clone(), default_output_handle, indicator_cache_data.clone());

                    // 发送到strategy
                    let strategy_output_handle = self.get_strategy_output_handle();
                    send_indicator_event(strategy_output_handle.output_handle_id.clone(), strategy_output_handle.clone(), indicator_cache_data);
                    }
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
                self.set_play_index(play_index_update_event.play_index).await;
                let strategy_output_handle = self.get_strategy_output_handle();
                let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: strategy_output_handle.output_handle_id.clone(),
                    play_index: self.get_play_index().await,
                    message_timestamp: get_utc8_timestamp_millis(),
                }));
                // 发送到strategy
                strategy_output_handle.send(signal).unwrap();
            }
        }
        Ok(())
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) -> Result<(), String> {
        tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        Ok(())
    }

}

impl IndicatorNodeContext {


    // 注册指标（初始化指标）向指标引擎发送注册请求
    pub async fn register_indicator_cache_key(&self) -> Result<bool, String> {

        let mut is_all_success = true;
        // 遍历已配置的指标，注册指标缓存键
        for indicator_cache_key in self.indicator_cache_keys.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();

            let register_indicator_params = AddCacheKeyParams {
                strategy_id: self.base_context.strategy_id.clone(),
                key: indicator_cache_key.clone().into(),
                max_size: None,
                duration: Duration::from_secs(30),
                sender: self.base_context.node_id.to_string(),
                timestamp: get_utc8_timestamp_millis(),
                responder: resp_tx,
            };
            let register_indicator_command = Command::CacheEngine(CacheEngineCommand::AddCacheKey(register_indicator_params));
            self.get_command_publisher().send(register_indicator_command).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.code() != 0 {
                is_all_success = false;
                break;
            }
        }

        Ok(is_all_success)
    }


    // 获取已经计算好的回测指标数据
    async fn get_backtest_indicator_cache(&self, indicator_cache_key: &BacktestIndicatorKey, play_index: i32) -> Result<Vec<Arc<CacheValue>>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetCacheParams {
            strategy_id: self.base_context.strategy_id.clone(),
            node_id: self.base_context.node_id.clone(),
            key: indicator_cache_key.clone().into(),
            index: Some(play_index as u32),
            limit: Some(1),
            sender: self.base_context.node_id.clone(),
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
                        return Ok(get_cache_data_response.cache_data)
                    }
                    _ => {
                        return Err(format!("节点{}收到回测K线缓存数据失败", self.base_context.node_id))
                    }
                }
            }
        }
        Err(format!("节点{}收到回测K线缓存数据失败", self.base_context.node_id))
    }

    // 计算指标(一次性将指标全部计算完成)
    pub async fn calculate_indicator(&self) -> Result<bool, String> {
        let mut is_all_success = true;
        for ind in self.backtest_config.exchange_mode_config.clone().unwrap().selected_indicators.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let params = CalculateBacktestIndicatorParams {
                strategy_id: self.base_context.strategy_id.clone(),
                node_id: self.base_context.node_id.clone(),
                kline_key: self.kline_cache_key.clone().into(),
                indicator_config: ind.indicator_config.clone(),
                sender: self.base_context.node_id.clone(),
                command_timestamp: get_utc8_timestamp_millis(),
                responder: resp_tx,
            };
            let calculate_indicator_command = Command::IndicatorEngine(IndicatorEngineCommand::CalculateBacktestIndicator(params));
            self.get_command_publisher().send(calculate_indicator_command).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.code() != 0 {
                is_all_success = false;
                break;
            }
        }
        Ok(is_all_success)
    }
}

