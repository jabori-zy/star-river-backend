pub mod start_node_context;
pub mod start_node_state_machine;

use tokio::sync::RwLock;
use std::sync::Arc;
use crate::strategy_engine::node::node_state_machine::BacktestNodeStateTransitionEvent;
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType};
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use super::start_node::start_node_state_machine::{StartNodeStateMachine,StartNodeStateAction};
use std::time::Duration;
use std::any::Any;
use crate::*;
use super::start_node::start_node_context::StartNodeContext;
use star_river_core::strategy::{BacktestStrategyConfig};
use event_center::communication::strategy::{StrategyCommandReceiver, NodeCommandSender};
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use tokio::sync::broadcast;
use virtual_trading::VirtualTradingSystem;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use star_river_core::error::engine_error::strategy_engine_error::node_error::BacktestStrategyNodeError;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::start_node_error::*;
use star_river_core::custom_type::{StrategyId, NodeId, NodeName};
use snafu::ResultExt;
// use start_node_log_message::*;
use super::node_message::common_log_message::*;
use super::node_message::start_node_log_message::*;
use event_center::event::strategy_event::NodeStateLogEvent;

#[derive(Debug)]
pub struct StartNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl Clone for StartNode {
    fn clone(&self) -> Self {
        StartNode {
            context: self.context.clone(),
        }
    }
}

impl StartNode {
    pub fn new(
        start_node_config: serde_json::Value,
        heartbeat: Arc<Mutex<Heartbeat>>,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver, // 策略内部事件接收器
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<i32>,
    ) -> Result<Self, StartNodeError> {
        let (strategy_id, node_id, node_name, backtest_strategy_config) =
            Self::check_start_node_config(start_node_config)?;

        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::StartNode,
            Box::new(StartNodeStateMachine::new(node_id.clone(), node_name.clone())),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx,
        );
        Ok(StartNode {
            context: Arc::new(RwLock::new(Box::new(StartNodeContext {
                base_context,
                node_config: Arc::new(RwLock::new(backtest_strategy_config)),
                heartbeat,
                virtual_trading_system,
                strategy_stats,
            }))),
        })
    }

    fn check_start_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, BacktestStrategyConfig), StartNodeError> {
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

        let backtest_strategy_config = serde_json::from_value::<BacktestStrategyConfig>(backtest_config_json)
            .context(ConfigDeserializationFailedSnafu {})?;

        // check initial balance (> 0)
        if backtest_strategy_config.initial_balance <= 0.0 {
            return ValueNotGreaterThanZeroSnafu {
                node_name: node_name.clone(),
                node_id: node_id.clone(),
                config_name: "initial balance".to_string(),
                config_value: backtest_strategy_config.initial_balance,
            }
            .fail();
        }

        // check leverage (> 0)
        if backtest_strategy_config.leverage <= 0 {
            return ValueNotGreaterThanZeroSnafu {
                node_name: node_name.clone(),
                node_id: node_id.clone(),
                config_name: "leverage".to_string(),
                config_value: backtest_strategy_config.leverage as f64,
            }
            .fail();
        }

        // check fee rate (>= 0)
        if backtest_strategy_config.fee_rate < 0.0 {
            return ValueNotGreaterThanOrEqualToZeroSnafu {
                node_name: node_name.clone(),
                node_id: node_id.clone(),
                config_name: "fee rate".to_string(),
                config_value: backtest_strategy_config.fee_rate,
            }
            .fail();
        }

        Ok((strategy_id, node_id, node_name, backtest_strategy_config))
    }
}

#[async_trait]
impl BacktestNodeTrait for StartNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }

    // get方法
    // 获取节点上下文
    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        let _from_node_id = from_node_id;
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        tracing::info!(node_id = %node_id, node_name = %node_name, "=================init start node====================");
        tracing::info!(node_id = %node_id, node_name = %node_name, "start init");
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize)
            .await?;

        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete)
            .await?;
        Ok(())
    }

    // 设置节点默认出口
    async fn set_output_handle(&mut self) {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;

        // 添加向strategy发送的出口(这个出口专门用来给strategy发送消息)
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!(
            "[{node_name}] setting strategy output handle: {}",
            strategy_output_handle_id
        );
        self.add_output_handle(strategy_output_handle_id, tx).await;

        // 添加默认出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let default_output_handle_id = format!("{}_default_output", node_id);
        tracing::debug!(
            "[{node_name}] setting default output handle: {}",
            default_output_handle_id
        );
        self.add_output_handle(default_output_handle_id, tx).await;
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let state = self.context.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop)
            .await
            .unwrap();
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete)
            .await
            .unwrap();
        Ok(())
    }

    async fn listen_node_events(&self) {}

    async fn update_node_state(
        &mut self,
        event: BacktestNodeStateTransitionEvent,
    ) -> Result<(), BacktestStrategyNodeError> {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let strategy_id = self.get_strategy_id().await;
        let strategy_output_handle = self.get_strategy_output_handle().await;

        let mut state_machine = self.get_state_machine().await;
        let transition_result = state_machine.transition(event)?;

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            // 克隆actions避免移动问题
            if let Some(start_action) = action.as_any().downcast_ref::<StartNodeStateAction>() {
                let current_state = state_machine.current_state();
                match start_action {
                    StartNodeStateAction::LogTransition => {
                        tracing::debug!(
                            "[{node_name}({node_id})] state transition: {:?} -> {:?}",
                            current_state,
                            transition_result.get_new_state()
                        );
                    }
                    StartNodeStateAction::ListenAndHandleInnerEvents => {
                        tracing::info!("[{node_name}({node_id})] starting to listen strategy inner events");
                        let log_message = ListenStrategyInnerEventsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            StartNodeStateAction::ListenAndHandleInnerEvents.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_strategy_inner_events().await;
                    }
                    StartNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("[{node_name}({node_id})] starting to listen strategy command");
                        let log_message = ListenStrategyCommandMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            StartNodeStateAction::ListenAndHandleStrategyCommand.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_strategy_command().await;
                    }
                    StartNodeStateAction::ListenAndHandlePlayIndex => {
                        tracing::info!("[{node_name}({node_id})] starting to listen play index change");
                        let log_message = ListenPlayIndexChangeMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            StartNodeStateAction::ListenAndHandlePlayIndex.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        self.listen_play_index_change().await;
                    }
                    StartNodeStateAction::InitVirtualTradingSystem => {
                        tracing::info!("[{node_name}({node_id})] start to init virtual trading system");
                        let log_message = InitVirtualTradingSystemMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            StartNodeStateAction::InitVirtualTradingSystem.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(start_node_context) = state_guard.as_any_mut().downcast_mut::<StartNodeContext>() {
                            start_node_context.init_virtual_trading_system().await;
                        }
                    }
                    StartNodeStateAction::InitStrategyStats => {
                        tracing::info!("[{node_name}({node_id})] start to init strategy stats");
                        let log_message = InitStrategyStatsMsg::new(node_id.clone(), node_name.clone());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            StartNodeStateAction::InitStrategyStats.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(start_node_context) = state_guard.as_any_mut().downcast_mut::<StartNodeContext>() {
                            start_node_context.init_strategy_stats().await;
                        }
                    }
                    StartNodeStateAction::LogNodeState => {
                        let log_message =
                            NodeStateLogMsg::new(node_id.clone(), node_name.clone(), current_state.to_string());
                        let log_event = NodeStateLogEvent::success(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            current_state.to_string(),
                            StartNodeStateAction::LogNodeState.to_string(),
                            log_message.to_string(),
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                    }
                    StartNodeStateAction::CancelAsyncTask => {
                        tracing::debug!("[{node_name}({node_id})] cancel async task");
                        self.cancel_task().await;
                    }
                    _ => {}
                }
                // 更新状态
                {
                    let mut state_guard = self.context.write().await;
                    state_guard.set_state_machine(state_machine.clone_box());
                }
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        Ok(())
    }
}

impl StartNode {
    // pub async fn send_finish_signal(&self, signal_index: i32) {
    //     let context = self.get_context();
    //     let mut state_guard = context.write().await;
    //     if let Some(start_node_context) =
    //         state_guard.as_any_mut().downcast_mut::<StartNodeContext>()
    //     {
    //         start_node_context.send_finish_signal(signal_index).await;
    //     }
    // }

    pub async fn listen_play_index_change(&self) {
        let (mut play_index_watch_rx, cancel_token, node_id) = {
            let context = self.get_context();
            let state_guard = context.read().await;
            let play_index_watch_rx = state_guard.get_play_index_watch_rx();
            let cancel_token = state_guard.get_cancel_token().clone();
            let node_id = state_guard.get_node_id().to_string();
            (play_index_watch_rx, cancel_token, node_id)
        };

        let context = self.get_context();

        // 节点接收播放索引变化
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 播放索引监听任务已中止", node_id);
                        break;
                    }
                    // 监听播放索引变化
                    receive_result = play_index_watch_rx.changed() => {
                        match receive_result {
                            Ok(_) => {
                                let state_guard = context.read().await;
                                let start_node_context = state_guard.as_ref().as_any().downcast_ref::<StartNodeContext>().unwrap();
                                start_node_context.handle_play_index().await;
                            }
                            Err(e) => {
                                // tracing::error!("节点{}监听播放索引错误: {}", node_id, e);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
