use std::sync::Arc;

use async_trait::async_trait;
use event_center::{EventCenterSingleton, event::Event};
use star_river_core::order::{FuturesOrderSide, OrderType};
use star_river_event::backtest_strategy::{
    node_event::{IndicatorNodeEvent, KlineNodeEvent, VariableNodeEvent},
    strategy_event::BacktestStrategyEvent,
};
use strategy_core::{
    event::{
        node_common_event::CommonEvent,
        strategy_event::{StrategyPerformanceUpdateEvent, StrategyRunningLogEvent},
    },
    strategy::context_trait::{
        StrategyBenchmarkExt, StrategyEventHandlerExt, StrategyIdentityExt, StrategyInfoExt, StrategyVariableExt, StrategyWorkflowExt,
    },
};
use strategy_stats::StrategyStatsEvent;
use virtual_trading::event::VtsEvent;

use super::BacktestStrategyContext;
use crate::{
    node::node_event::BacktestNodeEvent,
    strategy::{
        strategy_command::*,
        strategy_error::BacktestStrategyError,
        strategy_log_message::{
            FuturesOrderCanceledMsg, FuturesOrderCreatedMsg, FuturesOrderFilledMsg, LongLimitOrderExecutedDirectlyMsg,
            ShortLimitOrderExecutedDirectlyMsg,
        },
    },
};

#[async_trait]
impl StrategyEventHandlerExt for BacktestStrategyContext {
    type EngineEvent = Event;

    async fn handle_strategy_command(&mut self, command: BacktestStrategyCommand) {
        match command {
            // Get strategy cache keys
            BacktestStrategyCommand::GetStrategyKeys(cmd) => {
                let keys_map = self.keys().await;
                let keys = keys_map.keys().cloned().collect();
                let payload = GetStrategyKeysRespPayload::new(keys);
                let resp = GetStrategyKeysResponse::success(payload);
                cmd.respond(resp);
            }
            // Get minimum interval
            BacktestStrategyCommand::GetMinInterval(cmd) => {
                let min_interval = self.min_interval.clone();
                let payload = GetMinIntervalRespPayload::new(min_interval);
                let resp = GetMinIntervalResponse::success(payload);
                cmd.respond(resp);
            }
            BacktestStrategyCommand::InitKlineData(cmd) => {
                let result = self.init_kline_data(&cmd.kline_key, cmd.init_kline_data.clone()).await;
                match result {
                    Ok(()) => {
                        let payload = InitKlineDataRespPayload {};
                        let resp = InitKlineDataResponse::success(payload);
                        cmd.respond(resp);
                    }
                    Err(error) => {
                        let resp = InitKlineDataResponse::fail(Arc::new(error));
                        cmd.respond(resp);
                    }
                }
            }

            BacktestStrategyCommand::AppendKlineData(cmd) => {
                let result = self.append_kline_data(&cmd.kline_key, cmd.kline_series.clone()).await;
                match result {
                    Ok(()) => {
                        let payload = AppendKlineDataRespPayload {};
                        let resp = AppendKlineDataResponse::success(payload);
                        cmd.respond(resp);
                    }
                    Err(error) => {
                        let resp = AppendKlineDataResponse::fail(Arc::new(error));
                        cmd.respond(resp);
                    }
                }
            }

            BacktestStrategyCommand::GetKlineData(cmd) => {
                let result = self.get_kline_slice(cmd.datetime, cmd.index, &cmd.kline_key, cmd.limit).await;

                match result {
                    Ok(data) => {
                        let payload = GetKlineDataRespPayload::new(data.0, data.1);
                        let response = GetKlineDataResponse::success(payload);
                        cmd.respond(response);
                    }
                    Err(error) => {
                        let response = GetKlineDataResponse::fail(Arc::new(error));
                        cmd.respond(response);
                    }
                }
            }
            BacktestStrategyCommand::UpdateKlineData(cmd) => {
                let updated_kline = self.update_kline_data(&cmd.kline_key, &cmd.kline).await;
                let payload = UpdateKlineDataRespPayload::new(updated_kline);
                let response = UpdateKlineDataResponse::success(payload);
                cmd.respond(response);
            }

            BacktestStrategyCommand::InitIndicatorData(cmd) => {
                self.init_indicator_data(&cmd.indicator_key, cmd.indicator_series.clone()).await;
                let payload = InitIndicatorDataRespPayload {};
                let resp = InitIndicatorDataResponse::success(payload);
                cmd.respond(resp);
            }

            BacktestStrategyCommand::GetIndicatorData(cmd) => {
                let result_data = self
                    .get_indicator_slice(cmd.datetime, cmd.index, &cmd.indicator_key, cmd.limit)
                    .await;
                match result_data {
                    Ok(data) => {
                        let payload = GetIndicatorDataRespPayload::new(data.0, data.1);
                        let response = GetIndicatorDataResponse::success(payload);
                        cmd.respond(response);
                    }
                    Err(error) => {
                        let response = GetIndicatorDataResponse::fail(Arc::new(error));
                        cmd.respond(response);
                    }
                }
            }

            BacktestStrategyCommand::UpdateIndicatorData(cmd) => {
                let updated_indicator = self.update_indicator_data(&cmd.indicator_key, &cmd.indicator).await;
                let payload = UpdateIndicatorDataRespPayload::new(updated_indicator);
                let response = UpdateIndicatorDataResponse::success(payload);
                cmd.respond(response);
            }

            BacktestStrategyCommand::InitCustomVariableValue(cmd) => {
                self.init_custom_variables(cmd.custom_variables.clone()).await;
                let payload = InitCustomVarRespPayload {};
                let resp = InitCustomVarValueResponse::success(payload);
                cmd.respond(resp);
            }
            BacktestStrategyCommand::GetCustomVariableValue(cmd) => {
                let result = self.custom_variable(&cmd.var_name).await;
                if let Ok(value) = result {
                    let payload = GetCustomVarRespPayload::new(value);
                    let resp = GetCustomVarValueResponse::success(payload);
                    cmd.respond(resp);
                } else {
                    let err = result.unwrap_err();
                    let resp = GetCustomVarValueResponse::fail(Arc::new(err));
                    cmd.respond(resp);
                }
            }

            BacktestStrategyCommand::UpdateCustomVariableValue(cmd) => {
                let result = self
                    .update_custom_variable(
                        &cmd.update_var_config.var_name,
                        &cmd.update_var_config.update_var_value_operation,
                        cmd.update_var_config.update_operation_value.as_ref(),
                    )
                    .await;
                if let Ok(value) = result {
                    let payload = UpdateCustomVarRespPayload::new(value);
                    let resp = UpdateCustomVarValueResponse::success(payload);
                    cmd.respond(resp);
                } else {
                    let err = result.unwrap_err();
                    let resp = UpdateCustomVarValueResponse::fail(Arc::new(err));
                    cmd.respond(resp);
                }
            }

            BacktestStrategyCommand::ResetCustomVariableValue(cmd) => {
                let result = self.reset_custom_variable(&cmd.var_name).await;
                if let Ok(value) = result {
                    let payload = ResetCustomVarRespPayload::new(value);
                    let resp = ResetCustomVarValueResponse::success(payload);
                    cmd.respond(resp);
                } else {
                    let err = result.unwrap_err();
                    let resp = ResetCustomVarValueResponse::fail(Arc::new(err));
                    cmd.respond(resp);
                }
            }

            BacktestStrategyCommand::UpdateSysVariableValue(cmd) => {
                self.update_sys_variable(&cmd.sys_variable).await;
                let payload = UpdateSysVarRespPayload;
                let resp = UpdateSysVarValueResponse::success(payload);
                cmd.respond(resp);
            }

            BacktestStrategyCommand::AddNodeCycleTracker(cmd) => {
                let result = self.add_node_completed_cycle(cmd.node_id.clone(), cmd.cycle_tracker.clone()).await;
                if let Err(e) = result {
                    let resp = AddNodeCycleTrackerResponse::fail(Arc::new(e));
                    cmd.respond(resp);
                } else {
                    let payload = AddNodeCycleTrackerRespPayload;
                    let resp = AddNodeCycleTrackerResponse::success(payload);
                    cmd.respond(resp);
                }
            }
        }
    }

    async fn handle_engine_event(&mut self, _event: Event) {}

    // All node events are aggregated here
    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), BacktestStrategyError> {
        // Execution completed
        if let BacktestNodeEvent::Common(signal_event) = &node_event {
            match signal_event {
                // Execution finished
                CommonEvent::ExecuteOver(execute_over_event) => {
                    self.leaf_node_execution_completed(execute_over_event.node_id().clone());
                    let should_finalize = self.leaf_node_execution_tracker().is_all_completed();
                    tracing::debug!("======={}=======", self.cycle_id());
                    tracing::debug!(
                        "{} execute over. cycle_id: {}, node_id: {}, context: {:?}",
                        execute_over_event.node_name(),
                        execute_over_event.cycle_id(),
                        execute_over_event.node_id(),
                        execute_over_event.context
                    );
                    tracing::debug!(
                        "leaf node: {:?}. execution info: {:?}",
                        self.leaf_node_execution_tracker().leaf_node_ids,
                        self.leaf_node_execution_tracker().leaf_node_execution_info
                    );
                    if should_finalize {
                        tracing::debug!("should_finalize: {}", should_finalize);
                    }

                    // Step 2: If all leaf nodes are completed, perform cleanup and notification first, then record benchmark
                    if should_finalize {
                        {
                            let mut cycle_tracker_guard = self.cycle_tracker().write().await;
                            if let Some(cycle_tracker) = cycle_tracker_guard.as_mut() {
                                cycle_tracker.start_phase("execute_over");
                            }
                        }

                        // Clear execute_over_node_ids first

                        self.reset_leaf_node_execution_info();

                        // Notify waiting threads (includes more execution logic in benchmark)
                        self.execute_over_notify.notify_waiters();

                        // Step 3: End cycle tracker and record to benchmark (includes cleanup and notification time above)
                        let completed_tracker = {
                            let mut cycle_tracker_guard = self.cycle_tracker().write().await;
                            if let Some(cycle_tracker) = cycle_tracker_guard.as_mut() {
                                cycle_tracker.end_phase("execute_over");
                                let completed = cycle_tracker.end();
                                // Clear cycle_tracker
                                *cycle_tracker_guard = None;
                                Some(completed)
                            } else {
                                None
                            }
                        }; // cycle_tracker lock is released here

                        // If there's a completed tracker, add it to benchmark
                        if let Some(tracker) = completed_tracker {
                            {
                                let mut strategy_benchmark_guard = self.benchmark().write().await;
                                strategy_benchmark_guard.add_cycle_tracker(tracker);
                            }
                            let benchmark_clone = Arc::clone(&self.benchmark());

                            let strategy_id = self.strategy_id();
                            tokio::spawn(async move {
                                let strategy_benchmark_guard = benchmark_clone.read().await;
                                let report = strategy_benchmark_guard.report();
                                let event: BacktestStrategyEvent = StrategyPerformanceUpdateEvent::new(strategy_id, report.clone()).into();
                                let _ = EventCenterSingleton::publish(event.into()).await;
                            });
                        }
                    }
                }
                CommonEvent::NodeRunningLog(running_log_event) => {
                    self.add_running_log(running_log_event.clone()).await;
                    let backtest_strategy_event: BacktestStrategyEvent = running_log_event.clone().into();
                    let event: Event = backtest_strategy_event.into();
                    EventCenterSingleton::publish(event).await?;
                }
                CommonEvent::RunStateLog(state_log_event) => {
                    let backtest_strategy_event: BacktestStrategyEvent = state_log_event.clone().into();
                    let event: Event = backtest_strategy_event.into();
                    EventCenterSingleton::publish(event).await?;
                }
                _ => {}
            }
        }

        if let BacktestNodeEvent::KlineNode(kline_node_event) = &node_event {
            match kline_node_event {
                KlineNodeEvent::KlineUpdate(kline_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::KlineUpdate(kline_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await?;
                }
            }
        }

        if let BacktestNodeEvent::IndicatorNode(indicator_node_event) = &node_event {
            match indicator_node_event {
                IndicatorNodeEvent::IndicatorUpdate(indicator_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::IndicatorUpdate(indicator_update_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await?;
                }
            }
        }

        if let BacktestNodeEvent::VariableNode(variable_node_event) = &node_event {
            match variable_node_event {
                VariableNodeEvent::CustomVarUpdate(custom_variable_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::CustomVariableUpdate(custom_variable_update_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await?;
                }
                VariableNodeEvent::SysVarUpdate(sys_variable_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::SysVariableUpdate(sys_variable_update_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await?;
                }
            }
        }
        Ok(())
    }
}

impl BacktestStrategyContext {
    pub async fn handle_vts_event(&mut self, event: VtsEvent) -> Result<(), BacktestStrategyError> {
        match event {
            VtsEvent::LimitOrderExecutedDirectly { limit_price, order } => {
                let log_message = if order.order_side == FuturesOrderSide::Long {
                    LongLimitOrderExecutedDirectlyMsg::new(self.strategy_name().clone(), limit_price, order.open_price, order.order_id)
                        .to_string()
                } else {
                    ShortLimitOrderExecutedDirectlyMsg::new(self.strategy_name().clone(), limit_price, order.open_price, order.order_id)
                        .to_string()
                };

                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::warn_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message,
                    None,
                    None,
                    self.strategy_time(),
                )
                .into();
                EventCenterSingleton::publish(log_event.into()).await?;
            }
            VtsEvent::PositionCreated(position) => {
                let event = BacktestStrategyEvent::PositionCreated {
                    virtual_position: position,
                };
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::PositionUpdated(position) => {
                let event = BacktestStrategyEvent::PositionUpdated {
                    virtual_position: position,
                };
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::PositionClosed(position) => {
                let event = BacktestStrategyEvent::PositionClosed {
                    virtual_position: position,
                };
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::FuturesOrderCreated(order) => {
                if let OrderType::Limit = order.order_type {
                    let log_message = FuturesOrderCreatedMsg::new(
                        self.strategy_name().clone(),
                        order.order_id,
                        order.order_config_id,
                        order.open_price,
                        order.order_side.to_string(),
                    );
                    let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                        self.cycle_id(),
                        self.strategy_id().clone(),
                        log_message.to_string(),
                        order.to_value()?,
                        self.strategy_time(),
                    )
                    .into();
                    EventCenterSingleton::publish(log_event.into()).await?;
                }

                let event = BacktestStrategyEvent::FuturesOrderCreated { futures_order: order };

                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::FuturesOrderFilled(order) => {
                let log_message =
                    FuturesOrderFilledMsg::new(self.strategy_name().clone(), order.order_id, order.quantity, order.open_price);
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();
                let event = BacktestStrategyEvent::FuturesOrderFilled { futures_order: order };
                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::FuturesOrderCanceled(order) => {
                let log_message = FuturesOrderCanceledMsg::new(self.strategy_name().clone(), order.order_id);
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();

                let event = BacktestStrategyEvent::FuturesOrderCanceled { futures_order: order };
                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::TakeProfitOrderCreated(order) => {
                let log_message = FuturesOrderCreatedMsg::new(
                    self.strategy_name().clone(),
                    order.order_id,
                    order.order_config_id,
                    order.open_price,
                    order.order_side.to_string(),
                );
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();

                let event = BacktestStrategyEvent::TakeProfitOrderCreated { take_profit_order: order };

                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::TakeProfitOrderFilled(order) => {
                let log_message =
                    FuturesOrderFilledMsg::new(self.strategy_name().clone(), order.order_id, order.quantity, order.open_price);
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();
                let event = BacktestStrategyEvent::TakeProfitOrderFilled { take_profit_order: order };
                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::TakeProfitOrderCanceled(order) => {
                let log_message = FuturesOrderCanceledMsg::new(self.strategy_name().clone(), order.order_id);
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();
                let event = BacktestStrategyEvent::TakeProfitOrderCanceled { take_profit_order: order };
                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::StopLossOrderCreated(order) => {
                let log_message = FuturesOrderCreatedMsg::new(
                    self.strategy_name().clone(),
                    order.order_id,
                    order.order_config_id,
                    order.open_price,
                    order.order_side.to_string(),
                );
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();
                let event = BacktestStrategyEvent::StopLossOrderCreated { stop_loss_order: order };
                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::StopLossOrderFilled(order) => {
                let log_message =
                    FuturesOrderFilledMsg::new(self.strategy_name().clone(), order.order_id, order.quantity, order.open_price);
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();
                let event = BacktestStrategyEvent::StopLossOrderFilled { stop_loss_order: order };
                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::StopLossOrderCanceled(order) => {
                let log_message = FuturesOrderCanceledMsg::new(self.strategy_name().clone(), order.order_id);
                let log_event: BacktestStrategyEvent = StrategyRunningLogEvent::info_with_time(
                    self.cycle_id(),
                    self.strategy_id().clone(),
                    log_message.to_string(),
                    order.to_value()?,
                    self.strategy_time(),
                )
                .into();
                let event = BacktestStrategyEvent::StopLossOrderCanceled { stop_loss_order: order };
                EventCenterSingleton::publish(log_event.into()).await?;
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::TransactionCreated(transaction) => {
                let event = BacktestStrategyEvent::TransactionCreated { transaction: transaction };
                EventCenterSingleton::publish(event.into()).await?;
            }
            VtsEvent::UpdateFinished => {}
        }
        Ok(())
    }

    pub async fn handle_strategy_stats_event(&mut self, event: StrategyStatsEvent) -> Result<(), BacktestStrategyError> {
        match event {
            StrategyStatsEvent::StrategyStatsUpdated(snp_event) => {
                let event: BacktestStrategyEvent = snp_event.into();
                EventCenterSingleton::publish(event.into()).await?;
            }
        }
        Ok(())
    }
}
