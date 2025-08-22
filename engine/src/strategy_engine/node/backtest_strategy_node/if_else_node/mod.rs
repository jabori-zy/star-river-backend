mod if_else_node_state_machine;
mod if_else_node_context;
pub mod condition;
pub mod if_else_node_type;
mod utils;

use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use std::vec;
use async_trait::async_trait;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::RwLock;
use types::strategy::node_event::BacktestNodeEvent;
use event_center::EventPublisher;
use crate::strategy_engine::node::node_state_machine::*;
use std::time::Duration;
use super::if_else_node::if_else_node_state_machine::{IfElseNodeStateManager,IfElseNodeStateAction};
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use crate::strategy_engine::node::node_types::{NodeType,DefaultOutputHandleId};
use if_else_node_context::IfElseNodeContext;
use crate::strategy_engine::node::{NodeOutputHandle,BacktestNodeTrait};
use types::strategy::TradeMode;
use if_else_node_type::IfElseNodeBacktestConfig;
use event_center::{CommandPublisher, CommandReceiver, command::backtest_strategy_command::StrategyCommandReceiver};
use tokio::sync::Mutex;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::{StrategyInnerEventReceiver, StrategyInnerEventPublisher};
use types::custom_type::PlayIndex;


// 条件分支节点
#[derive(Debug, Clone)]
pub struct IfElseNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl IfElseNode {

    pub fn new(
        strategy_id: i32,
        node_id: String, 
        node_name: String,
        backtest_config: IfElseNodeBacktestConfig,
        event_publisher: EventPublisher,
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::IfElseNode,
            event_publisher,
            vec![],
            command_publisher,
            command_receiver,
            Box::new(IfElseNodeStateManager::new(BacktestNodeRunState::Created, node_id, node_name)),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(IfElseNodeContext {
                base_context,
                is_processing: false,
                received_flag: HashMap::new(),
                received_message: HashMap::new(),
                backtest_config,
            }))),
            
        }
    }

    async fn evaluate(&self) {
        let (node_id, cancel_token) = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let node_id = context_guard.get_node_id().clone();
            let cancel_token = context_guard.get_cancel_token().clone();
            (node_id, cancel_token)
        };
        
        
        let context = self.context.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点条件判断进程已中止", node_id);
                        break;
                    }
                    _ = async {
                        // 使用更短的锁持有时间
                        let should_evaluate = {
                            let context_guard = context.read().await; // 使用读锁检查状态
                            let if_else_node_context = context_guard
                                .as_any()
                                .downcast_ref::<IfElseNodeContext>()
                                .expect("转换为IfElseNodeContext失败");
                            
                            if_else_node_context.is_all_value_received()
                        };

                        if should_evaluate {
                            let mut context_guard = context.write().await; // 只有需要时才获取写锁
                            let if_else_node_context = context_guard
                                .as_any_mut()
                                .downcast_mut::<IfElseNodeContext>()
                                .expect("转换为IfElseNodeContext失败");
                            
                            // 双重检查，防止竞态条件
                            if if_else_node_context.is_all_value_received() {
                                tracing::debug!("{}: 所有值已接收，开始评估", node_id);
                                if_else_node_context.evaluate().await;
                                if_else_node_context.reset_received_flag();
                            }
                        }

                        // 动态调整sleep时间
                        let sleep_duration = if should_evaluate { 10 } else { 50 };
                        tokio::time::sleep(tokio::time::Duration::from_millis(sleep_duration)).await;
                    } => {}
                }
            }
        });

    }

}


#[async_trait]
impl BacktestNodeTrait for IfElseNode {

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }

    async fn set_output_handle(&mut self) {
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, strategy_output_handle_id = %strategy_output_handle_id, "setting strategy output handle");
        self.add_output_handle(strategy_output_handle_id, tx).await;

        // 添加默认出口
        // let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        // let default_output_handle_id = format!("{}_default_output", node_id); 
        // tracing::debug!(node_id = %node_id, node_name = %node_name, default_output_handle_id = %default_output_handle_id, "setting default output handle");
        // self.add_output_handle(default_output_handle_id, tx).await;

        // 添加else出口
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let else_output_handle_id = format!("{}_else_output", node_id); // else分支作为默认出口
        tracing::debug!(node_id = %node_id, node_name = %node_name, else_output_handle_id = %else_output_handle_id, "setting ELSE output handle");
        self.add_output_handle(else_output_handle_id, tx).await;

        let cases = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let if_else_node_context = context_guard.as_any().downcast_ref::<IfElseNodeContext>().unwrap();
            if_else_node_context.backtest_config.cases.clone()
        };

        for case in cases {
            let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            let case_id = case.case_id;
            let case_output_handle_id = format!("{}_output{}", node_id, case_id);
            self.add_output_handle(case_output_handle_id, tx).await;
        }
        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }


    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;


        Ok(())
        
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.context.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await?;
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = self.context.read().await.get_node_id().clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, state_manager) = {
            let mut state_manager = self.context.read().await.get_state_machine().clone_box();  // 使用读锁获取当前状态
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(if_else_node_state_action) = action.as_any().downcast_ref::<IfElseNodeStateAction>() {
                match if_else_node_state_action {
                    IfElseNodeStateAction::LogTransition => {
                        let current_state = self.context.read().await.get_state_machine().current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                }
                IfElseNodeStateAction::LogNodeState => {
                    let current_state = self.context.read().await.get_state_machine().current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }

                IfElseNodeStateAction::ListenAndHandleNodeEvents => {
                    tracing::info!("{}: 开始监听节点传递的message", node_id);
                    self.listen_node_events().await?;
                }
                IfElseNodeStateAction::ListenAndHandleInnerEvents => {
                    tracing::info!("{}: 开始监听策略内部事件", node_id);
                    self.listen_strategy_inner_events().await?;
                }
                IfElseNodeStateAction::InitReceivedData => {
                    tracing::info!("{}: 开始初始化接收标记", node_id);
                    let context = self.get_context();
                    let mut state_guard = context.write().await;
                    if let Some(if_else_node_context) = state_guard.as_any_mut().downcast_mut::<IfElseNodeContext>() {
                        if_else_node_context.init_received_data().await;
                    }
                }
                IfElseNodeStateAction::Evaluate => {
                    tracing::info!("{}: 开始判断条件", node_id);
                    self.evaluate().await;
                }
                IfElseNodeStateAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("{}: 开始监听策略命令", node_id);
                    self.listen_strategy_command().await?;
                }
                
                IfElseNodeStateAction::CancelAsyncTask => {
                    tracing::info!("{}: 开始取消异步任务", node_id);
                    self.cancel_task().await;
                }
                _ => {}
                // 所有动作执行完毕后更新节点最新的状态
                
            }
            // 所有动作执行完毕后更新节点最新的状态
            {
                self.context.write().await.set_state_machine(state_manager.clone_box());
            }
        }
    }
                    
        Ok(())
    }
}


// 比较操作符


