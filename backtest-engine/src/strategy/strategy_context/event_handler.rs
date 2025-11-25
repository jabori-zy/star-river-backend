use std::sync::Arc;

use async_trait::async_trait;
use event_center::{EventCenterSingleton, event::Event};
use star_river_event::backtest_strategy::{
    node_event::{FuturesOrderNodeEvent, IndicatorNodeEvent, KlineNodeEvent, PositionNodeEvent, VariableNodeEvent},
    strategy_event::BacktestStrategyEvent,
};
use strategy_core::{
    event::{node_common_event::CommonEvent, strategy_event::StrategyPerformanceUpdateEvent},
    strategy::context_trait::{
        StrategyBenchmarkExt, StrategyEventHandlerExt, StrategyIdentityExt, StrategyVariableExt, StrategyWorkflowExt,
    },
};

use super::BacktestStrategyContext;
use crate::{node::node_event::BacktestNodeEvent, strategy::strategy_command::*};

#[async_trait]
impl StrategyEventHandlerExt for BacktestStrategyContext {
    type EngineEvent = Event;

    async fn handle_strategy_command(&mut self, command: BacktestStrategyCommand) {
        match command {
            // 获取策略缓存keys
            BacktestStrategyCommand::GetStrategyKeys(cmd) => {
                let keys_map = self.keys().await;
                let keys = keys_map.keys().cloned().collect();
                let payload = GetStrategyKeysRespPayload::new(keys);
                let resp = GetStrategyKeysResponse::success(payload);
                cmd.respond(resp);
            }
            // 获取最小interval
            BacktestStrategyCommand::GetMinInterval(cmd) => {
                let min_interval = self.min_interval().clone();
                let payload = GetMinIntervalRespPayload::new(min_interval);
                let resp = GetMinIntervalResponse::success(payload);
                cmd.respond(resp);
            }
            BacktestStrategyCommand::InitKlineData(cmd) => {
                self.init_kline_data(&cmd.kline_key, cmd.init_kline_data.clone()).await;
                let payload = InitKlineDataRespPayload {};
                let resp = InitKlineDataResponse::success(payload);
                cmd.respond(resp);
            }

            BacktestStrategyCommand::AppendKlineData(cmd) => {
                self.append_kline_data(&cmd.kline_key, cmd.kline_series.clone()).await;
                let payload = AppendKlineDataRespPayload {};
                let resp = AppendKlineDataResponse::success(payload);
                cmd.respond(resp);
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

    // 所有节点发送的事件都会汇集到这里
    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // 执行完毕
        if let BacktestNodeEvent::Common(signal_event) = &node_event {
            match signal_event {
                // 执行结束
                CommonEvent::ExecuteOver(execute_over_event) => {
                    // tracing::debug!("execute_over_event: {:#?}", execute_over_event);
                    self.leaf_node_execution_completed(execute_over_event.node_id().clone());
                    let should_finalize = self.leaf_node_execution_tracker().is_all_completed();

                    tracing::debug!("{:#?}", self.leaf_node_execution_tracker());
                    tracing::debug!("should_finalize: {}", should_finalize);

                    // 第二步：如果所有叶子节点都完成，先执行清理和通知，再记录 benchmark
                    if should_finalize {
                        {
                            let mut cycle_tracker_guard = self.cycle_tracker().write().await;
                            if let Some(cycle_tracker) = cycle_tracker_guard.as_mut() {
                                cycle_tracker.start_phase("execute_over");
                            }
                        }

                        // 先清空 execute_over_node_ids

                        self.reset_leaf_node_execution_info();

                        // 通知等待的线程（包含更多执行逻辑在 benchmark 中）
                        self.execute_over_notify.notify_waiters();

                        // 第三步：结束 cycle tracker 并记录到 benchmark（包含了上面的清理和通知时间）
                        let completed_tracker = {
                            let mut cycle_tracker_guard = self.cycle_tracker().write().await;
                            if let Some(cycle_tracker) = cycle_tracker_guard.as_mut() {
                                cycle_tracker.end_phase("execute_over");
                                let completed = cycle_tracker.end();
                                // 清空 cycle_tracker
                                *cycle_tracker_guard = None;
                                Some(completed)
                            } else {
                                None
                            }
                        }; // cycle_tracker 锁在这里释放

                        // 如果有完成的 tracker，添加到 benchmark
                        if let Some(tracker) = completed_tracker {
                            {
                                let mut strategy_benchmark_guard = self.benchmark().write().await;
                                strategy_benchmark_guard.add_cycle_tracker(tracker);
                            }
                            let benchmark_clone = self.benchmark().clone();

                            let strategy_id = self.strategy_id();
                            tokio::task::spawn(async move {
                                let strategy_benchmark_guard = benchmark_clone.read().await;
                                let report = strategy_benchmark_guard.report();
                                let event: BacktestStrategyEvent = StrategyPerformanceUpdateEvent::new(strategy_id, report.clone()).into();
                                let _ = EventCenterSingleton::publish(event.into()).await;
                            });
                        }
                    }
                }
                CommonEvent::RunningLog(running_log_event) => {
                    self.add_running_log(running_log_event.clone()).await;
                    let backtest_strategy_event: BacktestStrategyEvent = running_log_event.clone().into();
                    let event: Event = backtest_strategy_event.into();
                    EventCenterSingleton::publish(event).await.unwrap();
                }
                CommonEvent::StateLog(state_log_event) => {
                    let backtest_strategy_event: BacktestStrategyEvent = state_log_event.clone().into();
                    let event: Event = backtest_strategy_event.into();
                    EventCenterSingleton::publish(event).await.unwrap();
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
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
            }
        }

        if let BacktestNodeEvent::IndicatorNode(indicator_node_event) = &node_event {
            match indicator_node_event {
                IndicatorNodeEvent::IndicatorUpdate(indicator_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::IndicatorUpdate(indicator_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
            }
        }

        if let BacktestNodeEvent::VariableNode(variable_node_event) = &node_event {
            match variable_node_event {
                VariableNodeEvent::CustomVarUpdate(custom_variable_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::CustomVariableUpdate(custom_variable_update_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                VariableNodeEvent::SysVarUpdate(sys_variable_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::SysVariableUpdate(sys_variable_update_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
            }
        }

        // 期货订单节点事件
        if let BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) = &node_event {
            match futures_order_node_event {
                FuturesOrderNodeEvent::FuturesOrderFilled(futures_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderFilled(futures_order_filled_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCreated(futures_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCreated(futures_order_created_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCanceled(futures_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCanceled(futures_order_canceled_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCreated(take_profit_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderCreated(take_profit_order_created_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCreated(stop_loss_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderCreated(stop_loss_order_created_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderFilled(take_profit_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderFilled(take_profit_order_filled_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderFilled(stop_loss_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderFilled(stop_loss_order_filled_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCanceled(take_profit_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderCanceled(take_profit_order_canceled_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCanceled(stop_loss_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderCanceled(stop_loss_order_canceled_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                FuturesOrderNodeEvent::TransactionCreated(transaction_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TransactionCreated(transaction_created_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
            }
        }

        if let BacktestNodeEvent::PositionManagementNode(position_management_node_event) = &node_event {
            match position_management_node_event {
                PositionNodeEvent::PositionCreated(position_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionCreated(position_created_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                PositionNodeEvent::PositionUpdated(position_updated_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionUpdated(position_updated_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                PositionNodeEvent::PositionClosed(position_closed_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionClosed(position_closed_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
            }
        }
    }

    // pub async fn handle_strategy_stats_event(&mut self, event: StrategyStatsEvent) -> Result<(), String> {
    //     match event {
    //         StrategyStatsEvent::StrategyStatsUpdated(strategy_stats_updated_event) => {
    //             // tracing::debug!("{}: 收到策略统计更新事件: {:?}", self.strategy_name, strategy_stats_updated_event);
    //             let backtest_strategy_event = BacktestStrategyEvent::StrategyStatsUpdated(strategy_stats_updated_event);
    //             EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
    //         }
    //     }
    //     Ok(())
    // }
}
