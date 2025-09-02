pub mod start_node_state_machine;
pub mod start_node_context;

use tokio::sync::RwLock;
use std::sync::Arc;
use event_center::EventPublisher;
use crate::strategy_engine::node::node_state_machine::BacktestNodeStateTransitionEvent;
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType};
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use super::start_node::start_node_state_machine::{StartNodeStateMachine,StartNodeStateAction};
use std::time::Duration;
use std::any::Any;
use crate::*;
use super::start_node::start_node_context::StartNodeContext;
use types::strategy::{LiveStrategyConfig, BacktestStrategyConfig, SimulatedConfig, TradeMode};
use event_center::{CommandPublisher, CommandReceiver, command::backtest_strategy_command::StrategyCommandReceiver};
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use std::collections::HashMap;
use types::strategy::strategy_inner_event::{StrategyInnerEventReceiver, StrategyInnerEventPublisher};
use types::strategy::node_event::BacktestNodeEvent;
use tokio::sync::broadcast;
use virtual_trading::VirtualTradingSystem;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use types::error::engine_error::strategy_engine_error::node_error::BacktestStrategyNodeError;

#[derive(Debug)]
pub struct StartNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>
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
        strategy_id: i32,
        node_id: String, 
        node_name: String,
        backtest_config: BacktestStrategyConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        heartbeat: Arc<Mutex<Heartbeat>>,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver, // 策略内部事件接收器
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_stats: Arc<RwLock<BacktestStrategyStats>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<i32>,
    ) -> Self {
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::StartNode,
            event_publisher,
            vec![],
            command_publisher,
            command_receiver,
            Box::new(StartNodeStateMachine::new(node_id.clone(), node_name.clone())),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx
        );
        StartNode {
            context: Arc::new(RwLock::new(Box::new(StartNodeContext {
                base_context,
                node_config: Arc::new(RwLock::new(backtest_config)),
                heartbeat,
                virtual_trading_system,
                strategy_stats,
            }))),
        }
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
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await?;

        tracing::info!(node_id = %node_id, node_name = %node_name, "init complete");
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    // 设置节点默认出口
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
        tracing::debug!(node_id = %node_id, node_name = %node_name, default_output_handle_id = %default_output_handle_id, "setting start node default output handle");
        self.add_output_handle(default_output_handle_id, tx).await;
    }

    

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        let state = self.context.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await.unwrap();
        
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn listen_node_events(&self) {}

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), BacktestStrategyNodeError> {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let (transition_result, state_manager) = {
            let node_guard = self.context.read().await;  // 使用读锁获取当前状态
            let mut state_manager = node_guard.get_state_machine().clone_box();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(start_action) = action.as_any().downcast_ref::<StartNodeStateAction>() {
                match start_action {
                StartNodeStateAction::LogTransition => {
                    let current_state = self.context.read().await.get_state_machine().current_state();
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "state transition: {:?} -> {:?}", current_state, transition_result.get_new_state());
                }
                StartNodeStateAction::ListenAndHandleInnerEvents => {
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "start listen strategy inner events");
                    self.listen_strategy_inner_events().await;
                }
                StartNodeStateAction::ListenAndHandleStrategyCommand => {
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "start listen strategy command");
                    self.listen_strategy_command().await;
                }
                StartNodeStateAction::ListenAndHandlePlayIndex => {
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "start listen play index");
                    self.listen_play_index_change().await;
                }
                StartNodeStateAction::InitVirtualTradingSystem => {
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "start init virtual trading system");
                    let context = self.get_context();
                    let mut state_guard = context.write().await;
                    if let Some(start_node_context) = state_guard.as_any_mut().downcast_mut::<StartNodeContext>() {
                        start_node_context.init_virtual_trading_system().await;
                    }
                }
                StartNodeStateAction::InitStrategyStats => {
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "start init strategy stats");
                    let context = self.get_context();
                    let mut state_guard = context.write().await;
                    if let Some(start_node_context) = state_guard.as_any_mut().downcast_mut::<StartNodeContext>() {
                        start_node_context.init_strategy_stats().await;
                    }
                }
                StartNodeStateAction::LogNodeState => {
                    let current_state = self.context.read().await.get_state_machine().current_state();
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "current state: {:?}", current_state);
                }
                StartNodeStateAction::CancelAsyncTask => {
                    tracing::debug!(node_id = %node_id, node_name = %node_name, "cancel async task");
                    self.cancel_task().await;
                }
                _ => {}
            }
            // 更新状态
            {
                let mut state_guard = self.context.write().await;
                state_guard.set_state_machine(state_manager.clone_box());
            }
        }
    }
        Ok(())
    }
}


impl StartNode {
    // pub async fn send_play_signal(&self) {
    //     let context = self.get_context();
    //     let mut state_guard = context.write().await;
    //     if let Some(start_node_context) = state_guard.as_any_mut().downcast_mut::<StartNodeContext>() {
    //         start_node_context.send_play_signal().await;
    //     }
    // }

    pub async fn send_finish_signal(&self, signal_index : i32) {
        let context = self.get_context();
        let mut state_guard = context.write().await;
        if let Some(start_node_context) = state_guard.as_any_mut().downcast_mut::<StartNodeContext>() {
            start_node_context.send_finish_signal(signal_index).await;
        }
    }


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