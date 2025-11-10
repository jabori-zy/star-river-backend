use async_trait::async_trait;
use event_center::{Event, EventCenterSingleton};
use event_center_core::communication::Response;
use key::{IndicatorKey, KeyTrait, KlineKey};
use star_river_core::kline::Kline;
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
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeEventHandlerExt, NodeHandleExt, NodeIdentityExt},
};
use ta_lib::{Indicator, IndicatorConfig};
use tokio::sync::oneshot;

use super::IndicatorNodeContext;
use crate::{
    node::{
        node_command::{BacktestNodeCommand, NodeResetRespPayload, NodeResetResponse},
        node_event::{BacktestNodeEvent, StartNodeEvent},
    },
    strategy::strategy_command::{
        GetMinIntervalSymbolsCmdPayload, GetMinIntervalSymbolsCommand, UpdateIndicatorDataCmdPayload, UpdateIndicatorDataCommand,
    },
};

#[async_trait]
impl NodeEventHandlerExt for IndicatorNodeContext {
    type EngineEvent = Event;

    async fn handle_node_command(&mut self, node_command: Self::NodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    self.kline_value.clear();
                    let payload = NodeResetRespPayload {};
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        match node_event {
            BacktestNodeEvent::KlineNode(kline_event) => {
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    let config_kline = self.node_config.exchange_mode_config.as_ref().unwrap().selected_symbol.clone();
                    if config_kline.symbol != kline_update_event.kline_key.get_symbol()
                        || config_kline.interval != kline_update_event.kline_key.get_interval()
                    {
                        return;
                    }
                    self.handle_kline_update(kline_update_event).await;
                }
            }
            _ => {}
        }
    }

    async fn handle_engine_event(&mut self, event: Self::EngineEvent) {
        tracing::info!("[{}] received engine event: {:?}", self.node_name(), event);
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
        play_index: i32,
        node_id: &String,
        node_name: &String,
        to_strategy: bool,
    ) {
        let payload = IndicatorUpdatePayload::new(
            indicator_key.get_exchange(),
            indicator_key.get_symbol(),
            indicator_key.get_interval(),
            config_id.clone(),
            indicator_key.get_indicator_config(),
            indicator_key.clone(),
            indicator_value,
            play_index,
        );
        let indicator_update_event: IndicatorNodeEvent =
            IndicatorUpdateEvent::new(node_id.clone(), node_name.clone(), handle_id.clone(), payload).into();

        // 发送到指标特定的输出handle（如果存在）
        let _ = self.output_handle_send(&handle_id, indicator_update_event.clone().into());

        // 发送到默认输出handle
        let _ = self.default_output_handle_send(indicator_update_event.clone().into());

        // 发送到strategy
        if to_strategy {
            let _ = self.strategy_bound_handle_send(indicator_update_event.into());
        }
    }

    // 处理k线更新事件
    pub(super) async fn handle_kline_update(&mut self, kline_update_event: KlineUpdateEvent) {
        let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);

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
                    self.send_trigger_event(output_handle_id).await;
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
                        self.send_indicator_update_event(
                            output_handle_id.clone(),
                            &indicator_key,
                            &config_id,
                            last_indicator.clone(),
                            kline_update_event.play_index,
                            &node_id,
                            &node_name,
                            true,
                        );
                    }
                } else {
                    // 发送触发事件
                    self.send_trigger_event(output_handle_id).await;
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
                let indicator_data = match self.get_indicator_data(&indicator_key, kline_update_event.play_index).await {
                    Ok(data) => data,
                    Err(e) => continue,
                };

                // 使用工具方法发送指标更新事件
                self.send_indicator_update_event(
                    output_handle_id.clone(),
                    &indicator_key,
                    &config_id,
                    indicator_data,
                    kline_update_event.play_index,
                    &node_id,
                    &node_name,
                    true,
                );
                cycle_tracker.end_phase(&phase_name);
            }
        }

        // 发送trigger事件
        let _ = self.send_execute_over_event();

        // 结束周期追踪并记录到 benchmark
        let completed_tracker = cycle_tracker.end();
        // tracing::debug!("{}", completed_tracker.get_cycle_report());
        self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
        // tracing::debug!("{}", self.benchmark.report());

        // ========== 调试示例 ==========
        // 方式1: 打印最近一个周期的详细报告（每个阶段的耗时和占比）
        // self.print_last_cycle_report();

        // 方式2: 每100个周期打印一次性能报告
        // if self.benchmark.get_total_cycles() % 100 == 0 {
        //     let report = self.get_performance_report();
        //     tracing::info!("\n{}", report);
        // }

        // 方式3: 检查性能异常
        // if let Some(warning) = self.check_performance_anomaly() {
        //     tracing::warn!("{}", warning);
        // }

        // 方式4: 获取最近5个周期的报告进行分析
        // let recent_reports = self.get_recent_cycle_reports(5);
        // for report in recent_reports {
        //     if let Some((slowest_phase, _)) = report.get_slowest_phase() {
        //         tracing::debug!("Play {}: slowest phase = {}", report.play_index, slowest_phase);
        //     }
        // }
    }

    pub async fn get_min_interval_symbols_from_strategy(&mut self) -> Result<Vec<KlineKey>, String> {
        let (tx, rx) = oneshot::channel();
        let payload = GetMinIntervalSymbolsCmdPayload {};
        let cmd = GetMinIntervalSymbolsCommand::new(self.node_id().clone(), tx, payload);

        let _ = self.send_strategy_command(cmd.into()).await;

        let response = rx.await.unwrap();
        match response {
            StrategyResponse::Success { payload, .. } => {
                return Ok(payload.keys.clone());
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
