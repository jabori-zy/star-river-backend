use super::super::backtest_strategy_context::BacktestStrategyContext;
use event_center::communication::strategy::NodeCommand;
use event_center::communication::strategy::backtest_strategy::command::BacktestNodeCommand;
use event_center::communication::strategy::backtest_strategy::response::{GetStrategyCacheKeysResponse, GetCurrentTimeResponse, GetMinIntervalSymbolsResponse};
use event_center::event::strategy_event::backtest_strategy_event::BacktestStrategyEvent;
use event_center::event::Event;
use event_center::EventCenterSingleton;
use event_center::event::node_event::backtest_node_event::futures_order_node_event::FuturesOrderNodeEvent;
use event_center::event::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::event::node_event::backtest_node_event::indicator_node_event::IndicatorNodeEvent;
use event_center::event::node_event::backtest_node_event::position_management_node_event::PositionManagementNodeEvent;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use star_river_core::strategy_stats::event::StrategyStatsEvent;
use event_center::event::node_event::backtest_node_event::CommonEvent;
use event_center::event::node_event::NodeEventTrait;



impl BacktestStrategyContext {
    pub async fn handle_node_command(&mut self, command: NodeCommand) -> Result<(), String> {
        match command {
            // 获取策略缓存keys
            NodeCommand::BacktestNode(BacktestNodeCommand::GetStrategyKeys(
                get_strategy_cache_keys_command,
            )) => {
                let keys_map = self.get_keys().await;
                let keys = keys_map.keys().cloned().collect();
                let get_strategy_cache_keys_response = GetStrategyCacheKeysResponse::success(keys);
                get_strategy_cache_keys_command
                    .responder
                    .send(get_strategy_cache_keys_response.into())
                    .unwrap();
            }
            // 获取当前时间
            NodeCommand::BacktestNode(BacktestNodeCommand::GetCurrentTime(
                get_current_time_command,
            )) => {
                let current_time = self.get_current_time().await;
                let get_current_time_response = GetCurrentTimeResponse::success(current_time);
                get_current_time_command
                    .responder
                    .send(get_current_time_response.into())
                    .unwrap();
            }
            // 获取最小时间间隔的symbol
            NodeCommand::BacktestNode(BacktestNodeCommand::GetMinIntervalSymbols(
                get_min_interval_symbols_command,
            )) => {
                let min_interval_symbols = self.get_min_interval_symbols();
                let get_min_interval_symbols_response = GetMinIntervalSymbolsResponse::success(min_interval_symbols);
                get_min_interval_symbols_command
                    .responder
                    .send(get_min_interval_symbols_response.into())
                    .unwrap();
            }
            _ => {}
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
                    // tracing::debug!("{}: 收到执行完毕事件: {:?}", self.strategy_name.clone(), execute_over_event);
                    // tracing::debug!("leaf_node_ids: {:#?}", self.leaf_node_ids);
                    let mut execute_over_node_ids = self.execute_over_node_ids.write().await;
                    if !execute_over_node_ids.contains(&execute_over_event.from_node_id()) {
                        execute_over_node_ids.push(execute_over_event.from_node_id().clone());
                    }

                    // 如果所有叶子节点都执行完毕，则通知等待的线程
                    if execute_over_node_ids.len() == self.leaf_node_ids.len() {
                        tracing::debug!(
                            "[{}]: notify waiting thread. leaf node ids: {:?}",
                            self.strategy_name.clone(),
                            execute_over_node_ids
                        );
                        self.execute_over_notify.notify_waiters();
                        // 通知完成后，清空execute_over_node_ids
                        execute_over_node_ids.clear();
                    }
                }
                CommonEvent::RunningLog(running_log_event) => {
                    self.add_running_log(running_log_event.clone()).await;
                    let backtest_strategy_event: BacktestStrategyEvent = running_log_event.clone().into();
                    let event: Event = backtest_strategy_event.into();
                    EventCenterSingleton::publish(event)
                        .await
                        .unwrap();
                }
                _ => {}
            }
        }

        if let BacktestNodeEvent::KlineNode(kline_node_event) = &node_event {
            match kline_node_event {
                KlineNodeEvent::KlineUpdate(kline_update_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::KlineUpdate(kline_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                KlineNodeEvent::StateLog(log_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::NodeStateLog(log_event.clone());
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
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
                    let backtest_strategy_event =
                        BacktestStrategyEvent::IndicatorUpdate(indicator_update_event.clone());
                    // tracing::debug!("backtest-strategy-context: {:?}", serde_json::to_string(&backtest_strategy_event).unwrap());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }

        // 期货订单节点事件
        if let BacktestNodeEvent::FuturesOrderNode(futures_order_node_event) = &node_event {
            match futures_order_node_event {
                FuturesOrderNodeEvent::FuturesOrderFilled(futures_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderFilled(
                        futures_order_filled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCreated(futures_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCreated(
                        futures_order_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCanceled(futures_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::FuturesOrderCanceled(
                        futures_order_canceled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCreated(take_profit_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderCreated(
                        take_profit_order_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCreated(stop_loss_order_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderCreated(
                        stop_loss_order_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderFilled(take_profit_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderFilled(
                        take_profit_order_filled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderFilled(stop_loss_order_filled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderFilled(
                        stop_loss_order_filled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCanceled(
                    take_profit_order_canceled_event,
                ) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TakeProfitOrderCanceled(
                        take_profit_order_canceled_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCanceled(stop_loss_order_canceled_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::StopLossOrderCanceled(
                        stop_loss_order_canceled_event.clone(),
                    );
                    //  let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TransactionCreated(transaction_created_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::TransactionCreated(
                        transaction_created_event.clone(),
                    );
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }

        if let BacktestNodeEvent::PositionManagementNode(position_management_node_event) =
            &node_event
        {
            match position_management_node_event {
                PositionManagementNodeEvent::PositionCreated(position_created_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::PositionCreated(position_created_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                PositionManagementNodeEvent::PositionUpdated(position_updated_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::PositionUpdated(position_updated_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                PositionManagementNodeEvent::PositionClosed(position_closed_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::PositionClosed(position_closed_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }
    }

    pub async fn handle_strategy_stats_event(
        &mut self,
        event: StrategyStatsEvent,
    ) -> Result<(), String> {
        match event {
            StrategyStatsEvent::StrategyStatsUpdated(strategy_stats_updated_event) => {
                // tracing::debug!("{}: 收到策略统计更新事件: {:?}", self.strategy_name, strategy_stats_updated_event);
                let backtest_strategy_event =
                    BacktestStrategyEvent::StrategyStatsUpdated(strategy_stats_updated_event);
                // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                EventCenterSingleton::publish(backtest_strategy_event.into())
                    .await
                    .unwrap();
            }
        }
        Ok(())
    }

}