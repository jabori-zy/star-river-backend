use event_center::communication::strategy::{GetIndicatorDataResponse, GetKlineDataResponse, InitIndicatorDataResponse, InitKlineDataResponse, NodeResponse, UpdateIndicatorDataResponse, UpdateKlineDataResponse};
use star_river_core::cache::CacheItem;
use super::{
    BacktestStrategyCommand, BacktestNodeEvent, BacktestStrategyContext, BacktestStrategyEvent, CommonEvent, Event,
    EventCenterSingleton, FuturesOrderNodeEvent, GetCurrentTimeResponse, GetMinIntervalSymbolsResponse,
    GetStrategyCacheKeysResponse, IndicatorNodeEvent, KlineNodeEvent, NodeCommand, NodeEventTrait,
    PositionManagementNodeEvent, StrategyStatsEvent,
};

impl BacktestStrategyContext {
    pub async fn handle_node_command(&mut self, command: NodeCommand) -> Result<(), String> {
        match command {
            // 获取策略缓存keys
            NodeCommand::BacktestNode(BacktestStrategyCommand::GetStrategyKeys(get_strategy_cache_keys_command)) => {
                let keys_map = self.get_keys().await;
                let keys = keys_map.keys().cloned().collect();
                let get_strategy_cache_keys_response = GetStrategyCacheKeysResponse::success(keys);
                get_strategy_cache_keys_command
                    .responder
                    .send(get_strategy_cache_keys_response.into())
                    .unwrap();
            }
            // 获取当前时间
            NodeCommand::BacktestNode(BacktestStrategyCommand::GetCurrentTime(get_current_time_command)) => {
                let current_time = self.get_current_time().await;
                let get_current_time_response = GetCurrentTimeResponse::success(current_time);
                get_current_time_command
                    .responder
                    .send(get_current_time_response.into())
                    .unwrap();
            }
            // 获取最小时间间隔的symbol
            NodeCommand::BacktestNode(BacktestStrategyCommand::GetMinIntervalSymbols(get_min_interval_symbols_command)) => {
                let min_interval_symbols = self.get_min_interval_symbols();
                let get_min_interval_symbols_response = GetMinIntervalSymbolsResponse::success(min_interval_symbols);
                get_min_interval_symbols_command
                    .responder
                    .send(get_min_interval_symbols_response.into())
                    .unwrap();
            }
            NodeCommand::BacktestNode(BacktestStrategyCommand::InitKlineData(command)) => {
                // 初始化k线数据
                let mut kline_data_guard = self.kline_data.write().await;
                if let Some(kline_data) = kline_data_guard.get(&command.kline_key) {
                    if kline_data.len() == 0 {
                        kline_data_guard.insert(command.kline_key, command.init_kline_data);
                    }
                } else {
                    kline_data_guard.insert(command.kline_key, command.init_kline_data);
                }

                let init_backtest_kline_data_response: NodeResponse = InitKlineDataResponse::success(command.node_id).into();
                command.responder.send(init_backtest_kline_data_response.into()).unwrap();
            }

            NodeCommand::BacktestNode(BacktestStrategyCommand::GetKlineData(command)) => {
                let kline_data_guard = self.kline_data.read().await;
                let result_data = if let Some(kline_data) = kline_data_guard.get(&command.kline_key) {
                    match (command.play_index, command.limit) {
                        // 有index，有limit
                        (Some(play_index), Some(limit)) => {
                            // 如果索引超出范围，返回空
                            if play_index as usize >= kline_data.len() {
                                Vec::new()
                            } else {
                                // 计算从索引开始向前取limit个元素
                                let end = play_index as usize + 1;
                                let start = if limit as usize >= end {
                                    0
                                } else {
                                    end - limit as usize
                                };
                                kline_data[start..end].to_vec()
                            }
                        },
                        // 有index，无limit
                        (Some(play_index), None) => {
                            // 如果索引超出范围，返回空
                            if play_index as usize >= kline_data.len() {
                                Vec::new()
                            } else {
                                // 从索引开始向前取所有元素（到开头）
                                let end = play_index as usize + 1;
                                kline_data[0..end].to_vec()
                            }
                        },
                        // 无index，有limit
                        (None, Some(limit)) => {
                            // 从后往前取limit条数据
                            if limit as usize >= kline_data.len() {
                                kline_data.clone()
                            } else {
                                let start = kline_data.len().saturating_sub(limit as usize);
                                kline_data[start..].to_vec()
                            }
                        },
                        // 无index，无limit
                        (None, None) => {
                            // 如果limit和index都为None，则返回所有数据
                            kline_data.clone()
                        }
                    }
                } else {
                    Vec::new()
                };

                let response = GetKlineDataResponse::success(result_data);
                command.responder.send(response.into()).unwrap();
            }
            NodeCommand::BacktestNode(BacktestStrategyCommand::UpdateKlineData(command)) => {
                // 先检查键是否存在，释放锁
                let key_exists = { self.kline_data.read().await.contains_key(&command.kline_key) };

                if !key_exists {
                    // 如果缓存键不存在，先初始化空的Vec
                    let mut kline_data_guard = self.kline_data.write().await;
                    kline_data_guard.insert(command.kline_key.clone(), Vec::new());
                }

                // 重新获取锁并更新
                let mut kline_data_guard = self.kline_data.write().await;
                let kline_data = kline_data_guard.get_mut(&command.kline_key).unwrap();

                if !key_exists || kline_data.len() == 0 {
                    // 判断是否为初始化
                    kline_data.clear();
                    kline_data.push(command.kline.clone());
                } else {
                    // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k线
                    if let Some(last_kline) = kline_data.last() {
                        if last_kline.get_timestamp() == command.kline.get_timestamp() {
                            kline_data.pop();
                            kline_data.push(command.kline.clone());
                        } else {
                            // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
                            kline_data.push(command.kline.clone());
                        }
                    } else {
                        kline_data.push(command.kline.clone());
                    }
                }

                // TODO: 创建并发送响应
                let response = UpdateKlineDataResponse::success(command.kline);
                command.responder.send(response.into()).unwrap();
            }

            NodeCommand::BacktestNode(BacktestStrategyCommand::InitIndicatorData(command)) => {
                // 初始化指标数据
                let mut indicator_data_guard = self.indicator_data.write().await;
                if let Some(indicator_data) = indicator_data_guard.get(&command.indicator_key) {
                    if indicator_data.len() == 0 {
                        indicator_data_guard.insert(command.indicator_key, command.indicator_series);
                    }
                } else {
                    indicator_data_guard.insert(command.indicator_key, command.indicator_series);
                }

                let init_indicator_data_response: NodeResponse = InitIndicatorDataResponse::success(command.node_id).into();
                command.responder.send(init_indicator_data_response.into()).unwrap();
            }

            NodeCommand::BacktestNode(BacktestStrategyCommand::GetIndicatorData(command)) => {
                let indicator_data_guard = self.indicator_data.read().await;
                let result_data = if let Some(indicator_data) = indicator_data_guard.get(&command.indicator_key) {
                    match (command.play_index, command.limit) {
                        // 有index，有limit
                        (Some(play_index), Some(limit)) => {
                            // 如果索引超出范围，返回空Vec
                            if play_index as usize >= indicator_data.len() {
                                Vec::new()
                            } else {
                                // 计算从索引开始向前取limit个元素
                                let end = play_index as usize + 1;
                                let start = if limit as usize >= end {
                                    0
                                } else {
                                    end - limit as usize
                                };
                                indicator_data[start..end].to_vec()
                            }
                        },
                        // 有index，无limit
                        (Some(play_index), None) => {
                            // 如果索引超出范围，返回空Vec
                            if play_index as usize >= indicator_data.len() {
                                Vec::new()
                            } else {
                                // 从索引开始向前取所有元素（到开头）
                                let end = play_index as usize + 1;
                                indicator_data[0..end].to_vec()
                            }
                        },
                        // 无index，有limit
                        (None, Some(limit)) => {
                            // 从后往前取limit条数据
                            if limit as usize >= indicator_data.len() {
                                indicator_data.clone()
                            } else {
                                let start = indicator_data.len().saturating_sub(limit as usize);
                                indicator_data[start..].to_vec()
                            }
                        },
                        // 无index，无limit
                        (None, None) => {
                            // 如果limit和index都为None，则返回所有数据
                            indicator_data.clone()
                        }
                    }
                } else {
                    Vec::new()
                };

                let response = GetIndicatorDataResponse::success(result_data);
                command.responder.send(response.into()).unwrap();
            }

            NodeCommand::BacktestNode(BacktestStrategyCommand::UpdateIndicatorData(command)) => {
                // 先检查键是否存在，释放锁
                let key_exists = { self.indicator_data.read().await.contains_key(&command.indicator_key) };

                if !key_exists {
                    // 如果缓存键不存在，先初始化空的Vec
                    let mut indicator_data_guard = self.indicator_data.write().await;
                    indicator_data_guard.insert(command.indicator_key.clone(), Vec::new());
                }

                // 重新获取锁并更新
                let mut indicator_data_guard = self.indicator_data.write().await;
                let indicator_data = indicator_data_guard.get_mut(&command.indicator_key).unwrap();

                if !key_exists || indicator_data.len() == 0 {
                    // 判断是否为初始化
                    indicator_data.clear();
                    indicator_data.push(command.indicator_data.clone());
                } else {
                    // 如果最新的一条数据时间戳等于最后一个指标的时间戳，则更新最后一条指标
                    if let Some(last_indicator) = indicator_data.last() {
                        if last_indicator.get_timestamp() == command.indicator_data.get_timestamp() {
                            indicator_data.pop();
                            indicator_data.push(command.indicator_data.clone());
                        } else {
                            // 如果最新的一条数据时间戳不等于最后一个指标的时间戳，则插入新数据
                            indicator_data.push(command.indicator_data.clone());
                        }
                    } else {
                        indicator_data.push(command.indicator_data.clone());
                    }
                }

                let response = UpdateIndicatorDataResponse::success(command.indicator_data);
                command.responder.send(response.into()).unwrap();
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
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                KlineNodeEvent::StateLog(log_event) => {
                    let backtest_strategy_event = BacktestStrategyEvent::NodeStateLog(log_event.clone());
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
                    let backtest_strategy_event =
                        BacktestStrategyEvent::FuturesOrderFilled(futures_order_filled_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCreated(futures_order_created_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::FuturesOrderCreated(futures_order_created_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::FuturesOrderCanceled(futures_order_canceled_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::FuturesOrderCanceled(futures_order_canceled_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCreated(take_profit_order_created_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::TakeProfitOrderCreated(take_profit_order_created_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCreated(stop_loss_order_created_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::StopLossOrderCreated(stop_loss_order_created_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderFilled(take_profit_order_filled_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::TakeProfitOrderFilled(take_profit_order_filled_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderFilled(stop_loss_order_filled_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::StopLossOrderFilled(stop_loss_order_filled_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TakeProfitOrderCanceled(take_profit_order_canceled_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::TakeProfitOrderCanceled(take_profit_order_canceled_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::StopLossOrderCanceled(stop_loss_order_canceled_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::StopLossOrderCanceled(stop_loss_order_canceled_event.clone());
                    //  let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
                FuturesOrderNodeEvent::TransactionCreated(transaction_created_event) => {
                    let backtest_strategy_event =
                        BacktestStrategyEvent::TransactionCreated(transaction_created_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }

        if let BacktestNodeEvent::PositionManagementNode(position_management_node_event) = &node_event {
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
                    let backtest_strategy_event = BacktestStrategyEvent::PositionClosed(position_closed_event.clone());
                    // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                    EventCenterSingleton::publish(backtest_strategy_event.into())
                        .await
                        .unwrap();
                }
            }
        }
    }

    pub async fn handle_strategy_stats_event(&mut self, event: StrategyStatsEvent) -> Result<(), String> {
        match event {
            StrategyStatsEvent::StrategyStatsUpdated(strategy_stats_updated_event) => {
                // tracing::debug!("{}: 收到策略统计更新事件: {:?}", self.strategy_name, strategy_stats_updated_event);
                let backtest_strategy_event = BacktestStrategyEvent::StrategyStatsUpdated(strategy_stats_updated_event);
                // let _ = self.event_publisher.publish(backtest_strategy_event.into()).await;
                EventCenterSingleton::publish(backtest_strategy_event.into())
                    .await
                    .unwrap();
            }
        }
        Ok(())
    }
}
