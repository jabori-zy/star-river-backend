use super::indicator_node_type::IndicatorNodeBacktestConfig;
use crate::strategy_engine::node::node_context::{
    BacktestBaseNodeContext, BacktestNodeContextTrait,
};
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use async_trait::async_trait;
use event_center::communication::engine::cache_engine::CacheEngineResponse;
use event_center::communication::engine::cache_engine::{AddCacheKeyParams, GetCacheParams};
use event_center::communication::engine::indicator_engine::CalculateBacktestIndicatorParams;
use event_center::communication::strategy::StrategyCommand;
use event_center::event::node_event::backtest_node_event::indicator_node_event::{
    IndicatorNodeEvent, IndicatorUpdateEvent,
};
use event_center::event::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::event::node_event::backtest_node_event::signal_event::{
    ExecuteOverEvent, SignalEvent,
};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::{event::Event, EventCenterSingleton};
use star_river_core::cache::key::{IndicatorKey, KlineKey};
use star_river_core::cache::{CacheValue, KeyTrait};

use event_center::communication::strategy::backtest_strategy::command::BacktestStrategyCommand;
use event_center::communication::strategy::backtest_strategy::command::NodeResetParams;
use event_center::communication::strategy::backtest_strategy::response::NodeResetResponse;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEvent;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::sync::RwLock;
use tokio::time::Duration;
use utils::get_utc8_timestamp_millis;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: IndicatorNodeBacktestConfig,
    pub is_registered: Arc<RwLock<bool>>,  // 是否已经注册指标
    pub kline_key: KlineKey,               // 回测K线缓存键
    pub indicator_keys: Vec<IndicatorKey>, // 指标缓存键
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
        self.base_context
            .output_handles
            .get(&default_output_handle_id)
            .unwrap()
            .clone()
    }

    async fn handle_event(&mut self, event: Event) {}

    async fn handle_node_event(&mut self, message: BacktestNodeEvent) {
        match message {
            BacktestNodeEvent::KlineNode(kline_event) => {
                // 提前获取配置信息，统一错误处理
                let exchange_config = self.backtest_config.exchange_mode_config.as_ref().unwrap();

                let current_play_index = self.get_play_index();

                // 如果索引不匹配，提前返回错误日志
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    tracing::debug!(
                        "{}: 接收到k线更新事件。事件的play_index: {}，节点的play_index: {}",
                        self.base_context.node_id,
                        kline_update_event.play_index,
                        current_play_index
                    );
                    tracing::debug!("is_leaf_node: {}", self.is_leaf_node());
                    if current_play_index != kline_update_event.play_index {
                        tracing::error!(
                            node_id = %self.base_context.node_id,
                            node_name = %self.base_context.node_name,
                            kline_cache_index = %kline_update_event.play_index,
                            signal_index = %current_play_index,
                            "kline cache index is not equal to signal index"
                        );
                        return;
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
                        let indicator_key = IndicatorKey {
                            exchange: exchange.clone(),
                            symbol: symbol.clone(),
                            interval: interval.clone(),
                            indicator_config: ind_config.indicator_config.clone(),
                            start_time: Some(time_range.start_date.to_string()),
                            end_time: Some(time_range.end_date.to_string()),
                        };

                        let from_handle_id = ind_config.output_handle_id.clone();

                        // 获取指标缓存数据，增加错误处理
                        let indicator_cache_data = match self
                            .get_backtest_indicator_cache(
                                &indicator_key,
                                kline_update_event.play_index,
                            )
                            .await
                        {
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
                        let send_indicator_event =
                            |handle_id: String,
                             output_handle: NodeOutputHandle,
                             data: Vec<Arc<CacheValue>>| {
                                let indicator_update_event = IndicatorUpdateEvent {
                                    from_node_id: node_id.clone(),
                                    from_node_name: node_name.clone(),
                                    from_handle_id: handle_id,
                                    exchange: indicator_key.get_exchange(),
                                    symbol: indicator_key.get_symbol(),
                                    interval: indicator_key.get_interval(),
                                    config_id: ind_config.config_id,
                                    indicator_config: indicator_key.get_indicator_config(),
                                    indicator_key: indicator_key.clone(),
                                    indicator_series: data,
                                    play_index: kline_update_event.play_index,
                                    timestamp: timestamp,
                                };

                                let event = BacktestNodeEvent::IndicatorNode(
                                    IndicatorNodeEvent::IndicatorUpdate(indicator_update_event),
                                );
                                // tracing::debug!("indicator-node-event: {:?}", serde_json::to_string(&event).unwrap());
                                let _ = output_handle.send(event);
                            };

                        // 发送到指标特定的输出handle
                        if let Some(output_handle) =
                            self.base_context.output_handles.get(&from_handle_id)
                        {
                            send_indicator_event(
                                from_handle_id,
                                output_handle.clone(),
                                indicator_cache_data.clone(),
                            );
                        }

                        // 发送到默认输出handle
                        let default_output_handle = self.get_default_output_handle();
                        send_indicator_event(
                            default_output_handle.output_handle_id.clone(),
                            default_output_handle,
                            indicator_cache_data.clone(),
                        );

                        // 发送到strategy
                        let strategy_output_handle = self.get_strategy_output_handle();
                        send_indicator_event(
                            strategy_output_handle.output_handle_id.clone(),
                            strategy_output_handle.clone(),
                            indicator_cache_data,
                        );
                    }
                    // 如果节点是叶子节点，则发送执行完毕事件
                    if self.is_leaf_node() {
                        let execute_over_event = ExecuteOverEvent {
                            from_node_id: self.get_node_id().clone(),
                            from_node_name: self.get_node_name().clone(),
                            from_node_handle_id: self.get_node_id().clone(),
                            play_index: self.get_play_index(),
                            timestamp: get_utc8_timestamp_millis(),
                        };
                        let strategy_output_handle = self.get_strategy_output_handle();
                        strategy_output_handle
                            .send(BacktestNodeEvent::Signal(SignalEvent::ExecuteOver(
                                execute_over_event,
                            )))
                            .unwrap();
                    }
                }
            }
            _ => {}
        }
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) {
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
            //     // 发送到strategy
            //     strategy_output_handle.send(signal).unwrap();
            // }
            StrategyInnerEvent::NodeReset => {
                tracing::info!("收到策略重置事件，清空指标缓存键");
            }
        }
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) {
        match strategy_command {
            StrategyCommand::BacktestStrategy(BacktestStrategyCommand::NodeReset(
                node_reset_params,
            )) => {
                if self.get_node_id() == &node_reset_params.node_id {
                    let response = NodeResetResponse::success(self.get_node_id().clone());
                    node_reset_params.responder.send(response.into()).unwrap();
                }
            }
            _ => {}
        }
    }
}

impl IndicatorNodeContext {
    // 注册指标（初始化指标）向指标引擎发送注册请求
    pub async fn register_indicator_cache_key(&self) -> Result<bool, String> {
        let mut is_all_success = true;
        // 遍历已配置的指标，注册指标缓存键
        for indicator_cache_key in self.indicator_keys.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();

            let register_indicator_params = AddCacheKeyParams::new(
                self.base_context.strategy_id.clone(),
                indicator_cache_key.clone().into(),
                None,
                Duration::from_secs(30),
                self.base_context.node_id.to_string(),
                resp_tx,
            );
            // self.get_command_publisher().send(register_indicator_command).await.unwrap();
            EventCenterSingleton::send_command(register_indicator_params.into())
                .await
                .unwrap();
            let response = resp_rx.await.unwrap();
            if !response.success() {
                is_all_success = false;
                break;
            }
        }

        Ok(is_all_success)
    }

    // 获取已经计算好的回测指标数据
    async fn get_backtest_indicator_cache(
        &self,
        indicator_cache_key: &IndicatorKey,
        play_index: i32,
    ) -> Result<Vec<Arc<CacheValue>>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_cache_params = GetCacheParams::new(
            self.base_context.strategy_id.clone(),
            self.base_context.node_id.clone(),
            indicator_cache_key.clone().into(),
            Some(play_index as u32),
            Some(1),
            self.base_context.node_id.clone(),
            resp_tx,
        );
        // self.get_command_publisher().send(get_cache_command.into()).await.unwrap();
        EventCenterSingleton::send_command(get_cache_params.into())
            .await
            .unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.success() {
            if let Ok(cache_reponse) = CacheEngineResponse::try_from(response) {
                match cache_reponse {
                    CacheEngineResponse::GetCacheData(get_cache_data_response) => {
                        return Ok(get_cache_data_response.cache_data)
                    }
                    _ => {
                        return Err(format!(
                            "节点{}收到回测K线缓存数据失败",
                            self.base_context.node_id
                        ))
                    }
                }
            }
        }
        Err(format!(
            "节点{}收到回测K线缓存数据失败",
            self.base_context.node_id
        ))
    }

    // 计算指标(一次性将指标全部计算完成)
    pub async fn calculate_indicator(&self) -> Result<bool, String> {
        let mut is_all_success = true;
        for ind in self
            .backtest_config
            .exchange_mode_config
            .clone()
            .unwrap()
            .selected_indicators
            .iter()
        {
            let (resp_tx, resp_rx) = oneshot::channel();
            let calculate_backtest_indicator_params = CalculateBacktestIndicatorParams::new(
                self.base_context.strategy_id.clone(),
                self.base_context.node_id.clone(),
                self.kline_key.clone().into(),
                ind.indicator_config.clone(),
                self.base_context.node_id.clone(),
                resp_tx,
            );
            EventCenterSingleton::send_command(calculate_backtest_indicator_params.into())
                .await
                .unwrap();
            let response = resp_rx.await.unwrap();
            if !response.success() {
                is_all_success = false;
                break;
            }
        }
        Ok(is_all_success)
    }
}
