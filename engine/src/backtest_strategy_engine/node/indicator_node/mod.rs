pub mod indicator_node_context;
pub mod indicator_node_state_machine;
pub mod indicator_node_type;

use crate::backtest_strategy_engine::node::kline_node::kline_node_type::SelectedSymbol;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_state_machine::*;
use crate::backtest_strategy_engine::node::{BacktestNodeTrait, NodeType};
use async_trait::async_trait;
use event_center::communication::backtest_strategy::{NodeCommandReceiver, StrategyCommandSender};
use indicator_node_context::IndicatorNodeContext;
use indicator_node_state_machine::{IndicatorNodeStateAction, IndicatorNodeStateManager};
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

use super::node_message::common_log_message::*;
use super::node_message::indicator_node_log_message::*;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::strategy_event::NodeStateLogEvent;
use indicator_node_type::IndicatorNodeBacktestConfig;
use indicator_node_type::{ExchangeModeConfig, SelectedIndicator};
use serde::de::IntoDeserializer;
use snafu::ResultExt;
use star_river_core::custom_type::PlayIndex;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::indicator_node_error::*;
use star_river_core::error::engine_error::strategy_engine_error::node_error::*;
use star_river_core::indicator::IndicatorConfig;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::strategy::deserialize_time_range;
use star_river_core::strategy::{BacktestDataSource, SelectedAccount};
use std::collections::HashMap;
use std::str::FromStr;

// 指标节点
#[derive(Debug, Clone)]
pub struct IndicatorNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl IndicatorNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
        node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, IndicatorNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) = Self::check_indicator_node_config(node_config)?;
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::IndicatorNode,
            Box::new(IndicatorNodeStateManager::new(BacktestNodeRunState::Created, node_id, node_name)),
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        );

        // 通过配置，获取指标缓存键
        let indicator_keys = Self::get_indicator_keys(&backtest_config);
        // 通过配置，获取回测K线缓存键
        let selected_kline_key = Self::get_kline_key(&backtest_config);

        let indicator_node_context = IndicatorNodeContext::new(base_context, backtest_config, selected_kline_key, indicator_keys);
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(indicator_node_context))),
        })
    }

    fn check_indicator_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, IndicatorNodeBacktestConfig), IndicatorNodeError> {
        let node_id = node_config
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "id".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_data = node_config
            .get("data")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "data".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_name = node_data
            .get("nodeName")
            .and_then(|name| name.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "nodeName".to_string(),
                }
                .build()
            })?
            .to_owned();
        let strategy_id = node_data
            .get("strategyId")
            .and_then(|id| id.as_i64())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "strategyId".to_string(),
                }
                .build()
            })?
            .to_owned() as StrategyId;

        let backtest_config_json = node_data
            .get("backtestConfig")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "backtestConfig".to_string(),
                }
                .build()
            })?
            .to_owned();

        let selected_account_json = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedAccount"))
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "selectedAccount".to_string(),
                }
                .build()
            })?
            .to_owned();
        let selected_account =
            serde_json::from_value::<SelectedAccount>(selected_account_json).context(ConfigDeserializationFailedSnafu {})?;

        let selected_symbol_json = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedSymbol"))
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "selectedSymbol".to_string(),
                }
                .build()
            })?
            .to_owned();
        let selected_symbol =
            serde_json::from_value::<SelectedSymbol>(selected_symbol_json).context(ConfigDeserializationFailedSnafu {})?;

        let time_range_json = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("timeRange"))
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "timeRange".to_string(),
                }
                .build()
            })?
            .to_owned();
        tracing::debug!("time_range_json: {:?}", time_range_json);
        let time_range = deserialize_time_range(time_range_json.into_deserializer()).context(ConfigDeserializationFailedSnafu {})?;
        tracing::debug!("time_range: {:?}", time_range);

        let data_source = backtest_config_json
            .get("dataSource")
            .and_then(|source| source.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "dataSource".to_string(),
                }
                .build()
            })?
            .to_owned();
        let data_source = BacktestDataSource::from_str(&data_source).context(DataSourceParseFailedSnafu { data_source })?;

        let selected_indicators_array = backtest_config_json
            .get("exchangeModeConfig")
            .and_then(|config| config.get("selectedIndicators"))
            .and_then(|indicators| indicators.as_array())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "selectedIndicators".to_string(),
                }
                .build()
            })?
            .to_owned();

        let mut selected_indicators = Vec::new();
        for ind_config in selected_indicators_array {
            let indicator_type = ind_config.get("indicatorType").and_then(|t| t.as_str()).ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "indicatorType".to_string(),
                }
                .build()
            })?;
            let indicator_config_json = ind_config
                .get("indicatorConfig")
                .ok_or_else(|| {
                    ConfigFieldValueNullSnafu {
                        field_name: "indicatorConfig".to_string(),
                    }
                    .build()
                })?
                .to_owned();
            let indicator_config = IndicatorConfig::new(indicator_type, &indicator_config_json)?;
            let config_id = ind_config
                .get("configId")
                .and_then(|id| id.as_i64())
                .ok_or_else(|| {
                    ConfigFieldValueNullSnafu {
                        field_name: "configId".to_string(),
                    }
                    .build()
                })?
                .to_owned() as i32;
            let output_handle_id = ind_config
                .get("outputHandleId")
                .and_then(|id| id.as_str())
                .ok_or_else(|| {
                    ConfigFieldValueNullSnafu {
                        field_name: "outputHandleId".to_string(),
                    }
                    .build()
                })?
                .to_owned();
            selected_indicators.push(SelectedIndicator {
                config_id,
                output_handle_id,
                indicator_config,
            });
        }
        let exchange_mode_config = ExchangeModeConfig {
            selected_account,
            selected_symbol,
            selected_indicators,
            time_range,
        };

        let backtest_config = IndicatorNodeBacktestConfig {
            data_source,
            exchange_mode_config: Some(exchange_mode_config),
            file_mode_config: None,
        };
        Ok((strategy_id, node_id, node_name, backtest_config))
    }

    fn get_indicator_keys(backtest_config: &IndicatorNodeBacktestConfig) -> HashMap<IndicatorKey, (i32, String)> {
        let exchange = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let symbol = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_symbol
            .symbol
            .clone();
        let interval = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_symbol
            .interval
            .clone();
        let time_range = backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

        let mut indicator_keys = HashMap::new();
        for indicator in backtest_config.exchange_mode_config.as_ref().unwrap().selected_indicators.iter() {
            let indicator_key = IndicatorKey {
                exchange: exchange.clone(),
                symbol: symbol.clone(),
                interval: interval.clone(),
                indicator_config: indicator.indicator_config.clone(),
                start_time: Some(time_range.start_date.to_string()),
                end_time: Some(time_range.end_date.to_string()),
            };
            indicator_keys.insert(indicator_key, (indicator.config_id, indicator.output_handle_id.clone()));
        }
        indicator_keys
    }

    fn get_kline_key(backtest_config: &IndicatorNodeBacktestConfig) -> KlineKey {
        let exchange = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
        let symbol = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_symbol
            .symbol
            .clone();
        let interval = backtest_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_symbol
            .interval
            .clone();
        let time_range = backtest_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

        let kline_key = KlineKey {
            exchange: exchange.clone(),
            symbol: symbol.clone(),
            interval: interval.clone(),
            start_time: Some(time_range.start_date.to_string()),
            end_time: Some(time_range.end_date.to_string()),
        };
        kline_key
    }
}

#[async_trait]
impl BacktestNodeTrait for IndicatorNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }

    // 设置节点的出口
    async fn set_output_handle(&mut self) {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;

        // 添加strategy_output_handle
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!("[{node_name}] setting strategy output handle: {}", strategy_output_handle_id);
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        self.add_output_handle(strategy_output_handle_id, tx).await;

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = format!("{}_default_output", node_id);
        tracing::debug!("[{node_name}] setting default output handle: {}", default_output_handle_id);
        self.add_output_handle(default_output_handle_id, tx).await;

        // 添加每一个indicator的出口
        let selected_indicator = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let indicator_node_context = context_guard.as_any().downcast_ref::<IndicatorNodeContext>().unwrap();
            let exchange_mode_config = indicator_node_context.backtest_config.exchange_mode_config.as_ref().unwrap();
            exchange_mode_config.selected_indicators.clone()
        };

        for indicator in selected_indicator.iter() {
            let indicator_output_handle_id = indicator.output_handle_id.clone();
            tracing::debug!("[{node_name}] setting indicator output handle: {}", indicator_output_handle_id);
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(indicator_output_handle_id, tx).await;
        }
        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await?;

        // 循环检查是否已经注册指标
        // 检查交易所是否注册成功，并且K线流是否订阅成功
        // loop {
        //     let is_registered = {
        //         let state_guard = self.context.read().await;
        //         let indicator_node_context = state_guard.as_any().downcast_ref::<IndicatorNodeContext>().unwrap();
        //         let is_registered = indicator_node_context.is_registered.read().await.clone();
        //         tracing::info!("{}: 检查是否已经注册指标: {}", self.get_node_id().await, is_registered);
        //         is_registered
        //     };
        //     if is_registered {
        //         break;
        //     }
        //     tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        // }

        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await?;

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete)
            .await
            .unwrap();
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), BacktestStrategyNodeError> {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let strategy_id = self.get_strategy_id().await;
        let strategy_output_handle = self.get_strategy_output_handle().await;

        // 获取状态管理器并执行转换
        let mut state_machine = self.get_state_machine().await; // 使用读锁获取当前状态
        let transition_result = state_machine.transition(event)?;

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(indicator_node_state_action) = action.as_any().downcast_ref::<IndicatorNodeStateAction>() {
                let current_state = state_machine.current_state();
                match indicator_node_state_action {
                    IndicatorNodeStateAction::LogTransition => {
                        tracing::info!(
                            "[{node_name}({node_id})] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    IndicatorNodeStateAction::LogNodeState => {
                        tracing::info!("[{node_name}({node_id})] current state: {:?}", current_state);
                        let log_message = NodeStateLogMsg::new(node_id.clone(), node_name.clone(), current_state.to_string());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IndicatorNodeStateAction::LogNodeState.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                    }
                    IndicatorNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("[{node_name}({node_id})] starting to listen external events");
                        let log_message = ListenExternalEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IndicatorNodeStateAction::ListenAndHandleExternalEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_external_events().await;
                    }
                    IndicatorNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("[{node_name}({node_id})] starting to listen node events");
                        let log_message = ListenNodeEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IndicatorNodeStateAction::ListenAndHandleNodeEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_node_events().await;
                    }
                    IndicatorNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("[{node_name}({node_id})] starting to listen strategy command");
                        let log_message = ListenStrategyCommandMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IndicatorNodeStateAction::ListenAndHandleStrategyCommand.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_strategy_command().await;
                    }

                    IndicatorNodeStateAction::InitIndicatorLookback => {
                        let context = self.get_context();
                        let mut context_guard = context.write().await;
                        let indicator_node_context = context_guard.as_any_mut().downcast_mut::<IndicatorNodeContext>().unwrap();
                        indicator_node_context.init_indicator_lookback().await;
                        tracing::info!("[{node_name})] init indicator lookback complete");
                    }

                    IndicatorNodeStateAction::GetMinIntervalSymbols => {
                        let context = self.get_context();

                        let mut context_guard = context.write().await;
                        if let Some(indicator_node_context) = context_guard.as_any_mut().downcast_mut::<IndicatorNodeContext>() {
                            let min_interval_symbols = indicator_node_context.get_min_interval_symbols().await.unwrap();
                            indicator_node_context.set_min_interval_symbols(min_interval_symbols);

                            let log_message = GetMinIntervalSymbolsSuccessMsg::new(node_id.clone(), node_name.clone());
                            let log_event = NodeStateLogEvent::success(
                                strategy_id.clone(),
                                node_id.clone(),
                                node_name.clone(),
                                current_state.to_string(),
                                IndicatorNodeStateAction::GetMinIntervalSymbols.to_string(),
                                log_message.to_string(),
                            );
                            let _ = strategy_output_handle.send(log_event.into());
                        }
                    }

                    IndicatorNodeStateAction::CalculateIndicator => {
                        tracing::info!("[{node_name}] starting to calculate indicator");
                        let log_message = CalculateIndicatorMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            IndicatorNodeStateAction::CalculateIndicator.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        let mut context = self.context.write().await;
                        let context = context.as_any_mut().downcast_mut::<IndicatorNodeContext>().unwrap();
                        let is_all_success = context.calculate_indicator().await.unwrap();

                        if is_all_success {
                            tracing::info!("[{node_name}] calculate indicator success");
                        } else {
                            tracing::error!("[{node_name}] calculate indicator failed");
                        }
                    }
                    IndicatorNodeStateAction::CancelAsyncTask => {
                        tracing::debug!("[{node_name}] cancel async task");
                        self.cancel_task().await;
                    }
                    _ => {}
                }
                // 所有动作执行完毕后更新节点最新的状态
                {
                    self.context.write().await.set_state_machine(state_machine.clone_box());
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
