use super::{
    BacktestNodeEvent, BacktestStrategyCommand, BacktestStrategyContext, BacktestStrategyEvent, Command, CommonEvent, Event,
    EventCenterSingleton, FuturesOrderNodeEvent, GetCurrentTimeResponse, GetMinIntervalSymbolsResponse, GetStrategyKeysResponse,
    IndicatorNodeEvent, KlineNodeEvent, NodeEventTrait, PositionManagementNodeEvent, StrategyStatsEvent,
};
use event_center::{
    communication::backtest_strategy::*, 
    event::{
        node_event::backtest_node_event::VariableNodeEvent, 
        strategy_event::backtest_strategy_event::StrategyPerformanceUpdateEvent
    }};
use std::sync::Arc;

impl BacktestStrategyContext {
    pub async fn handle_strategy_command(&mut self, command: BacktestStrategyCommand) -> Result<(), String> {
        match command {
            // 获取策略缓存keys
            BacktestStrategyCommand::GetStrategyKeys(cmd) => {
                let keys_map = self.get_keys().await;
                let keys = keys_map.keys().cloned().collect();
                let payload = GetStrategyKeysRespPayload::new(keys);
                let resp = GetStrategyKeysResponse::success(Some(payload));
                cmd.respond(resp);
            }
            // 获取当前时间
            BacktestStrategyCommand::GetCurrentTime(cmd) => {
                let current_time = self.get_current_time().await;
                let payload = GetCurrentTimeRespPayload::new(current_time);
                let resp = GetCurrentTimeResponse::success(Some(payload));
                cmd.respond(resp);
            }
            // 获取最小时间间隔的symbol
            BacktestStrategyCommand::GetMinIntervalSymbols(cmd) => {
                let min_interval_symbols = self.get_min_interval_symbols();
                let payload = GetMinIntervalSymbolsRespPayload::new(min_interval_symbols);
                let resp = GetMinIntervalSymbolsResponse::success(Some(payload));
                cmd.respond(resp);
            }
            BacktestStrategyCommand::InitKlineData(cmd) => {
                self.init_kline_data(&cmd.kline_key, cmd.init_kline_data.clone()).await;
                let resp = InitKlineDataResponse::success(None);
                cmd.respond(resp);
            }

            BacktestStrategyCommand::AppendKlineData(cmd) => {
                self.append_kline_data(&cmd.kline_key, cmd.kline_series.clone()).await;
                let resp = AppendKlineDataResponse::success(None);
                cmd.respond(resp);
            }

            BacktestStrategyCommand::GetKlineData(cmd) => {
                let result = self.get_kline_data(&cmd.kline_key, cmd.play_index, cmd.limit).await;
                match result {
                    Ok(data) => {
                        let payload = GetKlineDataRespPayload::new(data);
                        let response = GetKlineDataResponse::success(Some(payload));
                        cmd.respond(response);
                    }
                    Err(error) => {
                        let response = GetKlineDataResponse::error(Arc::new(error));
                        cmd.respond(response);
                    }
                }
            }
            BacktestStrategyCommand::UpdateKlineData(cmd) => {
                let updated_kline = self.update_kline_data(&cmd.kline_key, &cmd.kline).await;
                let payload = UpdateKlineDataRespPayload::new(updated_kline);
                let response = UpdateKlineDataResponse::success(Some(payload));
                cmd.respond(response);
            }

            BacktestStrategyCommand::InitIndicatorData(cmd) => {
                self.init_indicator_data(&cmd.indicator_key, cmd.indicator_series.clone()).await;
                let resp = InitIndicatorDataResponse::success(None);
                cmd.respond(resp);
            }

            BacktestStrategyCommand::GetIndicatorData(cmd) => {
                let result_data = self.get_indicator_data(&cmd.indicator_key, cmd.play_index, cmd.limit).await;
                let payload = GetIndicatorDataRespPayload::new(result_data);
                let response = GetIndicatorDataResponse::success(Some(payload));
                cmd.respond(response);
            }

            BacktestStrategyCommand::UpdateIndicatorData(cmd) => {
                let updated_indicator = self.update_indicator_data(&cmd.indicator_key, &cmd.indicator).await;
                let payload = UpdateIndicatorDataRespPayload::new(updated_indicator);
                let response = UpdateIndicatorDataResponse::success(Some(payload));
                cmd.respond(response);
            }

            BacktestStrategyCommand::InitCustomVariableValue(cmd) => {
                self.init_custom_variables(cmd.custom_variables.clone()).await;
                let resp = InitCustomVariableValueResponse::success(None);
                cmd.respond(resp);
            }
            BacktestStrategyCommand::GetCustomVariableValue(cmd) => {
                let result = self.get_custom_variable_value(cmd.var_name.clone()).await;
                if let Ok(value) = result {
                    let payload = GetCustomVariableRespPayload::new(value);
                    let resp = GetCustomVariableValueResponse::success(Some(payload));
                    cmd.respond(resp);
                } else {
                    let err = result.unwrap_err();
                    let resp = GetCustomVariableValueResponse::error(Arc::new(err));
                    cmd.respond(resp);
                }
            }

            BacktestStrategyCommand::UpdateCustomVariableValue(cmd) => {
                let result = self.update_custom_variable_value(&cmd.update_var_config).await;
                if let Ok(value) = result {
                    let payload = UpdateCustomVariableRespPayload::new(value);
                    let resp = UpdateCustomVariableValueResponse::success(Some(payload));
                    cmd.respond(resp);
                } else {
                    let err = result.unwrap_err();
                    let resp = UpdateCustomVariableValueResponse::error(Arc::new(err));
                    cmd.respond(resp);
                }
            }

            BacktestStrategyCommand::ResetCustomVariableValue(cmd) => {
                let result = self.reset_custom_variables(cmd.var_name.clone()).await;
                if let Ok(value) = result {
                    let payload = ResetCustomVariableRespPayload::new(value);
                    let resp = ResetCustomVariableValueResponse::success(Some(payload));
                    cmd.respond(resp);
                } else {
                    let err = result.unwrap_err();
                    let resp = ResetCustomVariableValueResponse::error(Arc::new(err));
                    cmd.respond(resp);
                }
            }

            BacktestStrategyCommand::UpdateSysVariableValue(cmd) => {
                self.update_sys_variable(&cmd.sys_variable).await;
                let resp = UpdateSysVariableValueResponse::success(None);
                cmd.respond(resp);
            }
            BacktestStrategyCommand::AddNodeCycleTracker(cmd) => {
                let result = self.add_node_cycle_tracker(cmd.node_id.clone(), cmd.cycle_tracker.clone()).await;
                if let Err(e) = result {
                    let resp = AddNodeCycleTrackerResponse::error(Arc::new(e));
                    cmd.respond(resp);
                } else {
                    let resp = AddNodeCycleTrackerResponse::success(None);
                    cmd.respond(resp);
                }
            }
        }
        Ok(())
    }

    async fn handle_event(&mut self, _event: Event) -> Result<(), String> {
        Ok(())
    }

    // 所有节点发送的事件都会汇集到这里
    pub async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // 执行完毕
        if let BacktestNodeEvent::Common(signal_event) = &node_event {
            match signal_event {
                // 执行结束
                CommonEvent::ExecuteOver(execute_over_event) => {
                    // tracing::debug!("leaf_node_ids: {:#?}", self.leaf_node_ids);

                    
                    // 第一步：快速更新 execute_over_node_ids 并检查是否所有叶子节点都完成
                    let should_finalize = {
                        let mut execute_over_node_ids = self.execute_over_node_ids.write().await;
                        if !execute_over_node_ids.contains(&execute_over_event.from_node_id()) {
                            execute_over_node_ids.push(execute_over_event.from_node_id().clone());
                        }
                        // 判断是否所有叶子节点都完成，然后立即释放锁
                        execute_over_node_ids.len() == self.leaf_node_ids.len()
                    }; // execute_over_node_ids 锁在这里释放

                    // 第二步：如果所有叶子节点都完成，先执行清理和通知，再记录 benchmark
                    if should_finalize {

                        {
                            let mut cycle_tracker_guard = self.cycle_tracker.write().await;
                            if let Some(cycle_tracker) = cycle_tracker_guard.as_mut() {
                                cycle_tracker.start_phase("execute_over");
                            }
                        }


                        // 先清空 execute_over_node_ids
                        {
                            let mut execute_over_node_ids = self.execute_over_node_ids.write().await;
                            execute_over_node_ids.clear();
                        } // execute_over_node_ids 锁在这里释放

                        // 通知等待的线程（包含更多执行逻辑在 benchmark 中）
                        self.execute_over_notify.notify_waiters();
                        
                        // 第三步：结束 cycle tracker 并记录到 benchmark（包含了上面的清理和通知时间）
                        let completed_tracker = {
                            let mut cycle_tracker_guard = self.cycle_tracker.write().await;
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
                                let mut strategy_benchmark_guard = self.benchmark.write().await;
                                strategy_benchmark_guard.add_cycle_tracker(tracker);
                            }
                            let benchmark_clone = self.benchmark.clone();

                            let strategy_id = self.strategy_id;
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
                KlineNodeEvent::StateLog(log_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::NodeStateLog(log_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                KlineNodeEvent::TimeUpdate(time_update_event) => {
                    // 更新策略的全局时间
                    self.set_current_time(time_update_event.current_time).await;
                }
                _ => {}
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
                VariableNodeEvent::CustomVariableUpdate(custom_variable_update_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::CustomVariableUpdate(custom_variable_update_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                VariableNodeEvent::SysVariableUpdate(sys_variable_update_event) => {
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
                PositionManagementNodeEvent::PositionCreated(position_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionCreated(position_created_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                PositionManagementNodeEvent::PositionUpdated(position_updated_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionUpdated(position_updated_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
                PositionManagementNodeEvent::PositionClosed(position_closed_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::PositionClosed(position_closed_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
                }
            }
        }
    }

    pub async fn handle_strategy_stats_event(&mut self, event: StrategyStatsEvent) -> Result<(), String> {
        match event {
            StrategyStatsEvent::StrategyStatsUpdated(strategy_stats_updated_event) => {
                // tracing::debug!("{}: 收到策略统计更新事件: {:?}", self.strategy_name, strategy_stats_updated_event);
                let backtest_strategy_event = BacktestStrategyEvent::StrategyStatsUpdated(strategy_stats_updated_event);
                EventCenterSingleton::publish(backtest_strategy_event.into()).await.unwrap();
            }
        }
        Ok(())
    }
}
