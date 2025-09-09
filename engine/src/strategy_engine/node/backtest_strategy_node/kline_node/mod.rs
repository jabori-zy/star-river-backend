pub mod kline_node_context;
pub mod kline_node_state_machine;
pub mod kline_node_type;

use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use tokio::sync::RwLock;
use std::sync::Arc;
use crate::strategy_engine::node::backtest_strategy_node::kline_node::kline_node_state_machine::KlineNodeStateChangeActions;
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType};
use crate::strategy_engine::node::node_state_machine::*;
use kline_node_state_machine::{KlineNodeStateMachine, KlineNodeStateAction};
use crate::strategy_engine::node::node_context::{BacktestNodeContextTrait,BacktestBaseNodeContext};
use kline_node_context::{KlineNodeContext};
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use event_center::command::backtest_strategy_command::StrategyCommandReceiver;
use kline_node_type::KlineNodeBacktestConfig;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::{StrategyInnerEventReceiver};
use types::strategy::node_event::BacktestNodeEvent;
use types::custom_type::PlayIndex;
use snafu::Report;
use types::error::engine_error::strategy_engine_error::node_error::*;
use types::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::kline_node_error::*;
use snafu::IntoError;
use types::strategy::node_event::NodeStateLogEvent;
use super::node_message::common_log_message::*;
use super::node_message::kline_node_log_message::*;
use types::custom_type::{StrategyId, NodeId, NodeName};
use snafu::ResultExt;

#[derive(Debug, Clone)]
pub struct KlineNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl KlineNode {
    pub fn new(
        node_config: serde_json::Value,
        heartbeat: Arc<Mutex<Heartbeat>>,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, KlineNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) =
            Self::check_kline_node_config(node_config)?;
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::KlineNode,
            Box::new(KlineNodeStateMachine::new(
                node_id,
                node_name,
                backtest_config.data_source.clone(),
            )),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx,
        );
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(KlineNodeContext {
                base_context,
                data_is_loaded: Arc::new(RwLock::new(false)),
                exchange_is_registered: Arc::new(RwLock::new(false)),
                backtest_config,
                heartbeat,
            }))),
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
            serde_json::from_value::<KlineNodeBacktestConfig>(kline_node_backtest_config)
                .context(ConfigDeserializationFailedSnafu {})?;

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
        tracing::debug!(node_id = %node_id, node_name = %node_name, strategy_output_handle_id = %strategy_output_handle_id, "setting strategy output handle");
        self.add_output_handle(strategy_output_handle_id, tx).await;

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = format!("{}_default_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, default_output_handle_id = %default_output_handle_id, "setting default output handle");
        self.add_output_handle(default_output_handle_id, tx).await;

        // 添加每一个symbol的出口
        let selected_symbols = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let kline_node_context = context_guard
                .as_any()
                .downcast_ref::<KlineNodeContext>()
                .unwrap();
            let exchange_mode_config = kline_node_context
                .backtest_config
                .exchange_mode_config
                .as_ref()
                .unwrap();
            exchange_mode_config.selected_symbols.clone()
        };

        for symbol in selected_symbols.iter() {
            let symbol_output_handle_id = symbol.output_handle_id.clone();
            tracing::debug!(node_id = %node_id, node_name = %node_name, symbol_output_handle_id = %symbol_output_handle_id, "setting symbol output handle");
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(symbol_output_handle_id, tx).await;
        }
        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!(
            "================={}====================",
            self.context.read().await.get_node_name()
        );
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        if let Err(error) = self
            .update_node_state(BacktestNodeStateTransitionEvent::Initialize)
            .await
        {
            let report = Report::from_error(&error);
            tracing::error!("report: {}", report.to_string());
            return Err(error);
        }
        tracing::info!(
            "{:?}: 初始化完成",
            self.context
                .read()
                .await
                .get_state_machine()
                .current_state()
        );

        // 检查交易所是否注册成功，并且K线流是否订阅成功
        loop {
            let is_registered_and_data_loaded = {
                let state_guard = self.context.read().await;
                let kline_node_context = state_guard
                    .as_any()
                    .downcast_ref::<KlineNodeContext>()
                    .unwrap();
                let is_registered = kline_node_context
                    .exchange_is_registered
                    .read()
                    .await
                    .clone();
                let is_data_loaded = kline_node_context.data_is_loaded.read().await.clone();
                is_registered && is_data_loaded
            };
            if is_registered_and_data_loaded {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete)
            .await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let state = self.get_context();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop)
            .await?;

        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete)
            .await?;
        Ok(())
    }

    async fn update_node_state(
        &mut self,
        event: BacktestNodeStateTransitionEvent,
    ) -> Result<(), BacktestStrategyNodeError> {
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
            if let Some(kline_node_state_action) =
                action.as_any().downcast_ref::<KlineNodeStateAction>()
            {
                let current_state = state_machine.current_state();

                match kline_node_state_action {
                    KlineNodeStateAction::LogTransition => {
                        tracing::debug!(
                            "[{node_name}({node_id})] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    KlineNodeStateAction::LogNodeState => {
                        let log_message = NodeStateLogMsg::new(
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                        );
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            KlineNodeStateAction::LogNodeState.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                    }
                    KlineNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!(
                            "[{node_name}({node_id})] starting to listen external events"
                        );
                        let log_message =
                            ListenExternalEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            KlineNodeStateAction::ListenAndHandleExternalEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_external_events().await;
                    }
                    KlineNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("[{node_name}({node_id})] starting to listen node events");
                        let log_message =
                            ListenNodeEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            KlineNodeStateAction::ListenAndHandleNodeEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_node_events().await;
                    }
                    KlineNodeStateAction::ListenAndHandleInnerEvents => {
                        tracing::info!(
                            "[{node_name}({node_id})] starting to listen strategy inner events"
                        );
                        let log_message =
                            ListenStrategyInnerEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            KlineNodeStateAction::ListenAndHandleInnerEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());

                        self.listen_strategy_inner_events().await;
                    }
                    KlineNodeStateAction::RegisterExchange => {
                        tracing::info!("[{node_name}({node_id})] start to register exchange");

                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) =
                            state_guard.as_any_mut().downcast_mut::<KlineNodeContext>()
                        {
                            // 1. send register exchange log
                            let exchange = kline_node_context
                                .backtest_config
                                .exchange_mode_config
                                .as_ref()
                                .unwrap()
                                .selected_account
                                .exchange
                                .clone();
                            let account_id = kline_node_context
                                .backtest_config
                                .exchange_mode_config
                                .as_ref()
                                .unwrap()
                                .selected_account
                                .account_id
                                .clone();
                            let log_message = StartRegisterExchangeMsg::new(
                                node_id.clone(),
                                node_name.clone(),
                                exchange.clone(),
                                account_id,
                            );
                            let log_event = NodeStateLogEvent::success(
                                strategy_id.clone(),
                                node_id.clone(),
                                node_name.clone(),
                                current_state.to_string(),
                                KlineNodeStateAction::RegisterExchange.to_string(),
                                log_message.to_string(),
                            );
                            let _ = strategy_output_handle.send(log_event.into());

                            // 2. register exchange
                            let response = kline_node_context.register_exchange().await.unwrap();
                            if response.success() {
                                *kline_node_context.exchange_is_registered.write().await = true;

                                let log_message = RegisterExchangeSuccessMsg::new(
                                    node_id.clone(),
                                    node_name.clone(),
                                    exchange,
                                    account_id,
                                );

                                let log_event = NodeStateLogEvent::success(
                                    strategy_id.clone(),
                                    node_id.clone(),
                                    node_name.clone(),
                                    current_state.to_string(),
                                    KlineNodeStateAction::RegisterExchange.to_string(),
                                    log_message.to_string(),
                                );
                                let _ = strategy_output_handle.send(log_event.into());
                            } else {
                                // 转换状态 Failed
                                let error = response.error();
                                let kline_error = RegisterExchangeSnafu {
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
                        tracing::info!(
                            "[{node_name}({node_id})] starting to load kline data from exchange"
                        );
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) =
                            state_guard.as_any_mut().downcast_mut::<KlineNodeContext>()
                        {
                            let is_all_success = kline_node_context
                                .load_kline_history_from_exchange()
                                .await
                                .unwrap();
                            if is_all_success {
                                // 加载K线历史成功后，设置data_is_loaded=true
                                *kline_node_context.data_is_loaded.write().await = true;
                                tracing::info!("[{node_name}({node_id})] load kline history from exchange success");
                            } else {
                                tracing::error!("[{node_name}({node_id})] load kline history from exchange failed");
                            }
                        }
                    }
                    KlineNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("[{node_name}({node_id})] start to listen strategy command");
                        self.listen_strategy_command().await;
                    }

                    KlineNodeStateAction::CancelAsyncTask => {
                        tracing::info!("[{node_name}({node_id})] cancel node task");
                        self.cancel_task().await;
                    }
                    KlineNodeStateAction::LogError(error) => {
                        tracing::error!("[{node_name}({node_id})] node failed: {:?}", error);
                    }
                    _ => {}
                }
            }
            // 动作执行完毕后更新节点最新的状态
            {
                self.context
                    .write()
                    .await
                    .set_state_machine(state_machine.clone_box());
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }
        Ok(())
    }
}
