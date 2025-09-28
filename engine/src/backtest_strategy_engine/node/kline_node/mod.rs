pub mod kline_node_context;
pub mod kline_node_state_machine;
pub mod kline_node_type;

use super::node_message::common_log_message::*;
use super::node_message::kline_node_log_message::*;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_state_machine::*;
use crate::backtest_strategy_engine::node::{BacktestNodeTrait, NodeType};
use async_trait::async_trait;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::{NodeCommandReceiver, StrategyCommandSender};
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::strategy_event::NodeStateLogEvent;
use heartbeat::Heartbeat;
use kline_node_context::KlineNodeContext;
use kline_node_state_machine::{KlineNodeStateAction, KlineNodeStateMachine};
use kline_node_type::KlineNodeBacktestConfig;
use snafu::IntoError;
use snafu::Report;
use snafu::ResultExt;
use star_river_core::custom_type::PlayIndex;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::kline_node_error::*;
use star_river_core::error::engine_error::strategy_engine_error::node_error::*;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::RwLock;
use tokio::sync::broadcast;
use super::node_utils::NodeUtils;

#[derive(Debug, Clone)]
pub struct KlineNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl KlineNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: StrategyCommandSender,
        node_command_receiver: Arc<Mutex<NodeCommandReceiver>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, KlineNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) = Self::check_kline_node_config(node_config)?;

        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::KlineNode,
            Box::new(KlineNodeStateMachine::new(node_id, node_name, backtest_config.data_source.clone())),
            strategy_command_sender,
            node_command_receiver,
            play_index_watch_rx,
        );
        let context = KlineNodeContext::new(base_context, backtest_config);
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(context))),
        })
    }

    fn check_kline_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, KlineNodeBacktestConfig), KlineNodeError> {
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
        let kline_node_backtest_config = node_data
            .get("backtestConfig")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "backtestConfig".to_string(),
                }
                .build()
            })?
            .to_owned();

        let backtest_strategy_config =
            serde_json::from_value::<KlineNodeBacktestConfig>(kline_node_backtest_config).context(ConfigDeserializationFailedSnafu {})?;

        Ok((strategy_id, node_id, node_name, backtest_strategy_config))
    }
}

#[async_trait]
impl BacktestNodeTrait for KlineNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }
    // 获取节点状态
    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }

    // 设置节点的出口
    async fn set_output_handle(&mut self) {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;

        // 添加向strategy发送的出口(这个出口专门用来给strategy发送消息)
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!("[{node_name}] setting strategy output handle: {}", strategy_output_handle_id);
        self.add_output_handle(strategy_output_handle_id, tx).await;

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = format!("{}_default_output", node_id);
        tracing::debug!("[{node_name}] setting default output handle: {}", default_output_handle_id);
        self.add_output_handle(default_output_handle_id, tx).await;

        // 添加每一个symbol的出口
        let selected_symbols = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let kline_node_context = context_guard.as_any().downcast_ref::<KlineNodeContext>().unwrap();
            let exchange_mode_config = kline_node_context.backtest_config.exchange_mode_config.as_ref().unwrap();
            exchange_mode_config.selected_symbols.clone()
        };

        for symbol in selected_symbols.iter() {
            let symbol_output_handle_id = symbol.output_handle_id.clone();
            tracing::debug!("[{node_name}] setting symbol output handle: {}", symbol_output_handle_id);
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(symbol_output_handle_id, tx).await;
        }
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        if let Err(error) = self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await {
            let report = Report::from_error(&error);
            tracing::error!("report: {}", report.to_string());
            return Err(error);
        }
        
        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let state = self.get_context();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await?;

        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await?;
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), BacktestStrategyNodeError> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let strategy_id = self.get_strategy_id().await;
        let strategy_output_handle = self.get_strategy_output_handle().await;

        // 获取状态管理器并执行转换
        let mut state_machine = self.get_state_machine().await; // 使用读锁获取当前状态
        let transition_result = state_machine.transition(event)?;

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            // 克隆actions避免移动问题
            if let Some(kline_node_state_action) = action.as_any().downcast_ref::<KlineNodeStateAction>() {
                let current_state = state_machine.current_state();

                match kline_node_state_action {
                    KlineNodeStateAction::LogTransition => {
                        tracing::debug!(
                            "[{node_name}] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    KlineNodeStateAction::LogNodeState => {
                        let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeStateAction::LogNodeState.to_string(), &strategy_output_handle).await;
                    }
                    KlineNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("[{node_name}] starting to listen external events");
                        let log_message = ListenExternalEventsMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeStateAction::ListenAndHandleExternalEvents.to_string(), &strategy_output_handle).await;
                        
                        self.listen_external_events().await;
                    }
                    KlineNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("[{node_name}] starting to listen node events");
                        let log_message = ListenNodeEventsMsg::new(node_name.clone());
                        NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeStateAction::ListenAndHandleNodeEvents.to_string(), &strategy_output_handle).await;
                        
                        self.listen_node_events().await;
                    }
                    KlineNodeStateAction::GetMinIntervalSymbols => {
                        tracing::info!("[{node_name}] start to get min interval symbols");
                        let context = self.get_context();

                        let mut context_guard = context.write().await;
                        if let Some(kline_node_context) = context_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            let min_interval_symbols = kline_node_context.get_min_interval_symbols().await.unwrap();
                            kline_node_context.set_min_interval_symbols(min_interval_symbols);

                            let log_message = GetMinIntervalSymbolsSuccessMsg::new(node_name.clone());
                            NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeStateAction::GetMinIntervalSymbols.to_string(), &strategy_output_handle).await;
                        }
                    }
                    KlineNodeStateAction::RegisterExchange => {
                        tracing::info!("[{node_name}] start to register exchange");

                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) = state_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            // 1. send register exchange log
                            let exchange = kline_node_context
                                .backtest_config
                                .exchange_mode_config
                                .as_ref()
                                .unwrap()
                                .selected_account
                                .exchange
                                .clone();
                            let log_message = StartRegisterExchangeMsg::new(node_name.clone(), exchange.clone());
                            NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeStateAction::RegisterExchange.to_string(), &strategy_output_handle).await;
                            

                            // 2. register exchange
                            let response = kline_node_context.register_exchange().await.unwrap();
                            if response.is_success() {
                                let log_message = RegisterExchangeSuccessMsg::new(node_name.clone(), exchange);
                                NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeStateAction::RegisterExchange.to_string(), &strategy_output_handle).await;

                                
                            } else {
                                // 转换状态 Failed
                                let error = response.get_error();
                                let kline_error = RegisterExchangeFailedSnafu {
                                    node_id: node_id.clone(),
                                    node_name: node_name.clone(),
                                }
                                .into_error(error.clone());

                                let log_event = NodeStateLogEvent::error(
                                    strategy_id.clone(),
                                    node_id.clone(),
                                    node_name.clone(),
                                    BacktestNodeRunState::Failed.to_string(),
                                    KlineNodeStateAction::RegisterExchange.to_string(),
                                    &kline_error,
                                );
                                let _ = strategy_output_handle.send(log_event.into());
                                return Err(kline_error.into());
                            }
                        }
                    }
                    KlineNodeStateAction::LoadHistoryFromExchange => {
                        tracing::info!("[{node_name}] starting to load kline data from exchange");
                        let context = self.get_context();
                        let mut context_guard = context.write().await;
                        if let Some(kline_node_context) = context_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            let load_result = kline_node_context.load_kline_history_from_exchange().await;
                            match load_result {
                                Ok(()) => {
                                    tracing::info!("[{node_name}] load kline history from exchange success");
                                    let log_message = LoadKlineDataSuccessMsg::new(node_name.clone());
                                    NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeStateAction::LoadHistoryFromExchange.to_string(), &strategy_output_handle).await;
                                }
                                Err(e) => {
                                    NodeUtils::send_error_status_event(strategy_id, node_id.clone(), node_name.clone(), KlineNodeStateAction::LoadHistoryFromExchange.to_string(), &e, &strategy_output_handle).await;
                                    return Err(e.into());
                                }
                            }
                            tracing::info!("[{node_name}] load kline history from exchange success");  
                        }
                    }
                    KlineNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("[{node_name}] start to listen strategy command");
                        self.listen_strategy_command().await;
                    }

                    KlineNodeStateAction::CancelAsyncTask => {
                        tracing::info!("[{node_name}] cancel node task");
                        self.cancel_task().await;
                    }
                    KlineNodeStateAction::LogError(error) => {
                        tracing::error!("[{node_name}] node failed: {:?}", error);
                    }
                    _ => {}
                }
            }
            // 动作执行完毕后更新节点最新的状态
            {
                self.context.write().await.set_state_machine(state_machine.clone_box());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
