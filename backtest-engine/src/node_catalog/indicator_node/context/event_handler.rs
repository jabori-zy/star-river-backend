use async_trait::async_trait;
use event_center::{Event, EventCenterSingleton};
use event_center_core::communication::Response;
use key::{IndicatorKey, KeyTrait, KlineKey};
use star_river_core::kline::{Kline, KlineInterval};
use star_river_event::{
    backtest_strategy::node_event::{
        IndicatorNodeEvent, KlineNodeEvent,
        indicator_node_event::{IndicatorUpdateEvent, IndicatorUpdatePayload},
        kline_node_event::KlineUpdateEvent,
    },
    communication::{CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorCommand, IndicatorEngineCommand},
};
use strategy_core::{
    benchmark::node_benchmark::CycleTracker,
    communication::strategy::StrategyResponse,
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeInfoExt, NodeRelationExt},
};
use ta_lib::{Indicator, IndicatorConfig};
use tokio::sync::oneshot;

use super::IndicatorNodeContext;
use crate::{
    node::{
        node_command::{BacktestNodeCommand, NodeResetRespPayload, NodeResetResponse},
        node_error::IndicatorNodeError,
        node_event::BacktestNodeEvent,
    },
    strategy::strategy_command::{
        GetMinIntervalCmdPayload, GetMinIntervalCommand, UpdateIndicatorDataCmdPayload, UpdateIndicatorDataCommand,
    },
};

#[async_trait]
impl NodeEventHandlerExt for IndicatorNodeContext {
    type EngineEvent = Event;
    type Error = IndicatorNodeError;

    async fn handle_command(&mut self, node_command: Self::NodeCommand) -> Result<(), IndicatorNodeError> {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    self.kline_value.clear();
                    let payload = NodeResetRespPayload {};
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    async fn handle_source_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), IndicatorNodeError> {
        match node_event {
            BacktestNodeEvent::KlineNode(kline_event) => {
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    let config_kline = self.node_config.exchange_mode_config.as_ref().unwrap().selected_symbol.clone();
                    if config_kline.symbol != kline_update_event.kline_key.symbol()
                        || config_kline.interval != kline_update_event.kline_key.interval()
                    {
                        return Ok(());
                    }
                    self.handle_kline_update(kline_update_event).await;
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }

    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), IndicatorNodeError> {
        Ok(())
    }
}

impl IndicatorNodeContext {
    /// 发送指标更新事件的工具方法
    fn send_indicator_update_event(
        &self,
        handle_id: String,
        indicator_key: &IndicatorKey,
        config_id: &i32,
        indicator_value: Indicator,
        node_id: &String,
        node_name: &String,
    ) -> Result<(), crate::node::node_error::BacktestNodeError> {
        // 事件生成闭包
        let generate_event = |target_handle_id: String| {
            let payload = IndicatorUpdatePayload::new(
                indicator_key.exchange(),
                indicator_key.symbol(),
                indicator_key.interval(),
                config_id.clone(),
                indicator_key.get_indicator_config(),
                indicator_key.clone(),
                indicator_value.clone(),
            );
            let indicator_update_event: IndicatorNodeEvent = IndicatorUpdateEvent::new_with_time(
                self.cycle_id(),
                node_id.clone(),
                node_name.clone(),
                target_handle_id,
                self.strategy_time(),
                payload,
            )
            .into();
            let backtest_node_event: BacktestNodeEvent = indicator_update_event.into();
            backtest_node_event
        };

        // 渠道1: 发送到策略绑定输出句柄
        let strategy_output_handle = self.strategy_bound_handle();
        let event = generate_event(strategy_output_handle.output_handle_id().clone());
        self.strategy_bound_handle_send(event)?;

        // 渠道2: 根据节点类型发送到符号特定输出句柄
        if self.is_leaf_node() {
            self.send_execute_over_event(Some(*config_id), Some(self.strategy_time()))?;
        } else {
            let event = generate_event(handle_id.clone());
            self.output_handle_send(event)?;
        }

        // 渠道3: 发送到默认输出句柄
        let default_output_handle = self.default_output_handle().unwrap();
        let event = generate_event(default_output_handle.output_handle_id().clone());
        self.default_output_handle_send(event)?;

        Ok(())
    }

    // 处理k线更新事件
    pub(super) async fn handle_kline_update(&mut self, kline_update_event: KlineUpdateEvent) {
        let mut cycle_tracker = CycleTracker::new(self.cycle_id());

        // 提取公共数据
        let node_id = self.node_id().clone();
        let node_name = self.node_name().clone();
        let kline_key = kline_update_event.kline_key.clone();

        let indicator_keys = self.indicator_keys.clone();

        // 如果当前k线key不是最小周期交易对，则更新指标缓存数据
        if !self.min_interval_symbols.contains(&self.selected_kline_key) {
            for (indicator_key, (config_id, output_handle_id)) in indicator_keys.iter() {
                // 开始追踪当前指标的计算阶段
                let phase_name = format!("calculate indicator {}", config_id);
                cycle_tracker.start_phase(&phase_name);

                self.update_kline_data(indicator_key.clone(), kline_update_event.kline.clone())
                    .await;

                let kline_series = self.kline_value.get(indicator_key).unwrap();
                let lookback = self.indicator_lookback.get(indicator_key).unwrap();
                if kline_series.len() < *lookback + 1 {
                    if self.is_leaf_node() {
                        self.send_execute_over_event(Some(*config_id), Some(self.strategy_time())).unwrap();
                    } else {
                        self.send_trigger_event(output_handle_id, Some(self.strategy_time())).await.unwrap();
                    }
                    cycle_tracker.end_phase(&phase_name);
                    continue;
                }
                let calculate_reuslt = self
                    .request_calculate_indicator(&kline_key, kline_series, &indicator_key.indicator_config)
                    .await;
                if let Ok(indicator_data) = calculate_reuslt {
                    // 更新指标
                    let last_indicator = indicator_data.last().unwrap();
                    let update_result = self.update_strategy_indciator_data(indicator_key, last_indicator.clone()).await;
                    if let Ok(()) = update_result {
                        // 使用工具方法发送指标更新事件
                        if let Err(e) = self.send_indicator_update_event(
                            output_handle_id.clone(),
                            &indicator_key,
                            &config_id,
                            last_indicator.clone(),
                            &node_id,
                            &node_name,
                        ) {
                            tracing::error!(
                                node_id = %self.node_id(),
                                node_name = %self.node_name(),
                                "Failed to send indicator update event: {}", e
                            );
                        }
                    }
                } else {
                    // 发送触发事件
                    self.send_trigger_event(output_handle_id, Some(self.strategy_time())).await.unwrap();
                }

                // 结束当前指标的追踪
                cycle_tracker.end_phase(&phase_name);
            }
        }
        // 如果当前k线key是最小周期交易对，则直接发送指标更新事件
        else {
            // 遍历指标缓存键，从策略中获取指标数据
            for (indicator_key, (config_id, output_handle_id)) in self.indicator_keys.iter() {
                let phase_name = format!("get indicator data {}", config_id);
                cycle_tracker.start_phase(&phase_name);
                // 获取指标缓存数据，增加错误处理
                let indicator_data = match self.get_indicator_data(&indicator_key).await {
                    Ok(data) => data,
                    Err(_) => continue,
                };

                // 使用工具方法发送指标更新事件
                if let Err(e) = self.send_indicator_update_event(
                    output_handle_id.clone(),
                    &indicator_key,
                    &config_id,
                    indicator_data,
                    &node_id,
                    &node_name,
                ) {
                    tracing::error!(
                        node_id = %self.node_id(),
                        node_name = %self.node_name(),
                        "Failed to send indicator update event: {}", e
                    );
                }
                cycle_tracker.end_phase(&phase_name);
            }
        }

        // 结束周期追踪并记录到 benchmark
        let completed_tracker = cycle_tracker.end();
        // tracing::debug!("{}", completed_tracker.get_cycle_report());
        self.mount_node_cycle_tracker(self.node_id().clone(), self.node_name().clone(), completed_tracker)
            .await
            .unwrap();
        // tracing::debug!("{}", self.benchmark.report());
    }

    pub async fn init_min_interval_from_strategy(&mut self) -> Result<KlineInterval, String> {
        let (tx, rx) = oneshot::channel();
        let payload = GetMinIntervalCmdPayload {};
        let cmd = GetMinIntervalCommand::new(self.node_id().clone(), tx, payload);

        let _ = self.send_strategy_command(cmd.into()).await;

        let response = rx.await.unwrap();
        match response {
            StrategyResponse::Success { payload, .. } => {
                return Ok(payload.interval);
            }
            StrategyResponse::Fail { .. } => {
                return Err("获取最小周期交易对失败".to_string());
            }
        }
    }

    // 请求计算指标
    pub async fn request_calculate_indicator(
        &self,
        kline_key: &KlineKey,
        kline_series: &Vec<Kline>,
        indicator_config: &IndicatorConfig,
    ) -> Result<Vec<Indicator>, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = CalculateHistoryIndicatorCmdPayload::new(
            self.strategy_id().clone(),
            self.node_id().clone(),
            kline_key.clone(),
            kline_series.clone(),
            indicator_config.clone(),
        );
        let cmd: IndicatorEngineCommand = CalculateHistoryIndicatorCommand::new(self.node_id().clone(), resp_tx, payload).into();
        let _ = EventCenterSingleton::send_command(cmd.into()).await;

        let response = resp_rx.await.unwrap();
        match response {
            Response::Success { payload, .. } => {
                return Ok(payload.indicators.clone());
            }
            Response::Fail { .. } => {
                return Err("计算指标失败".to_string());
            }
        }
    }

    pub async fn update_strategy_indciator_data(&self, indicator_key: &IndicatorKey, lastest_indicator: Indicator) -> Result<(), String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = UpdateIndicatorDataCmdPayload::new(indicator_key.clone(), lastest_indicator.clone());
        let cmd = UpdateIndicatorDataCommand::new(self.node_id().clone(), resp_tx, payload);

        let _ = self.send_strategy_command(cmd.into()).await;
        let response = resp_rx.await.unwrap();
        match response {
            StrategyResponse::Success { .. } => {
                return Ok(());
            }
            StrategyResponse::Fail { .. } => {
                return Err("更新指标数据失败".to_string());
            }
        }
    }
}
