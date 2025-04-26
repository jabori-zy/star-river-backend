mod if_else_node_state_machine;
mod if_else_node_context;
pub mod condition;

use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use std::vec;
use async_trait::async_trait;
use tokio::sync::broadcast;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;
use types::strategy::message::NodeMessage;
use event_center::EventPublisher;
use super::NodeStateTransitionEvent;
use std::time::Duration;
use super::if_else_node::if_else_node_state_machine::{IfElseNodeStateManager,IfElseNodeStateAction};
use super::node_context::{BaseNodeContext,NodeContext};
use super::node_types::{NodeType,DefaultOutputHandleId};
use condition::*;
use if_else_node_context::IfElseNodeContext;
use super::{NodeRunState,NodeOutputHandle,NodeTrait};
use types::strategy::TradeMode;


// 条件分支节点
#[derive(Debug, Clone)]
pub struct IfElseNode {
    pub context: Arc<RwLock<Box<dyn NodeContext>>>,
}

impl IfElseNode {

    pub fn new(
        strategy_id: i64,
        node_id: String, 
        node_name: String, 
        cases: Vec<Case>, 
        trade_mode: TradeMode,
        event_publisher: EventPublisher,
    ) -> Self {
        let base_context = BaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            trade_mode,
            NodeType::IfElseNode,
            event_publisher,
            vec![],
            Box::new(IfElseNodeStateManager::new(NodeRunState::Created, node_id, node_name)),
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(IfElseNodeContext {
                base_context,
                current_batch_id: None,
                is_processing: false,
                received_flag: HashMap::new(),
                received_value: HashMap::new(),
                cases,
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
                        // 检查状态并获取需要的数据
                        let should_evaluate = {
                            // 如果所有节点都已接收数据，则返回true
                            let mut context_guard = context.write().await;
                            let if_else_node_context = context_guard.as_any_mut().downcast_mut::<IfElseNodeContext>().expect("转换为IfElseNodeContext失败");

                            if !if_else_node_context.received_flag.values().all(|flag| *flag) {
                                    false
                                } else {
                                    // 重置标记位
                                    for flag in if_else_node_context.received_flag.values_mut() {
                                        *flag = false;
                                    
                                }
                                true
                            }
                        
                        };

                        if should_evaluate {
                            let mut context_guard = context.write().await;
                            let if_else_node_context = context_guard.as_any_mut().downcast_mut::<IfElseNodeContext>().expect("转换为IfElseNodeContext失败");
                            if_else_node_context.evaluate().await;
                        }

                        // 添加短暂延迟避免过度循环
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    } => {}
                }
            }
        });

    }

}


#[async_trait]
impl NodeTrait for IfElseNode {

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn NodeContext>>> {
        self.context.clone()
    }

    async fn set_output_handle(&mut self) {
        tracing::debug!("{}: 设置节点默认出口", self.get_node_id().await);
        let node_id = self.get_node_id().await;
        let (tx, _) = broadcast::channel::<NodeMessage>(100);

        self.add_output_handle(DefaultOutputHandleId::IfElseNodeElseOutput.to_string(), tx).await;

        let context = self.get_context();
        let mut state_guard = context.write().await;
        if let Some(if_else_node_context) = state_guard.as_any_mut().downcast_mut::<IfElseNodeContext>() {
            let cases = if_else_node_context.cases.clone();
            for case in cases {
                let (tx, _) = broadcast::channel::<NodeMessage>(100);
                let handle = NodeOutputHandle {
                    node_id: node_id.clone(),
                    handle_id: format!("if_else_node_case_{}_output", case.case_id),
                    sender: tx,
                    connect_count: 0,
                };

                if_else_node_context.get_output_handle_mut().insert(format!("if_else_node_case_{}_output", case.case_id), handle);
            }
        }
        tracing::debug!("{}: 设置节点默认出口成功: {}", node_id, DefaultOutputHandleId::IfElseNodeElseOutput.to_string());
    }


    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_run_state(NodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_run_state(NodeStateTransitionEvent::InitializeComplete).await?;


        Ok(())
        
    }

    async fn start(&mut self) -> Result<(), String> {
        let state = self.context.clone();
        tracing::info!("{}: 开始启动", state.read().await.get_node_id());
        // 切换为starting状态
        self.update_run_state(NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;

        // 切换为running状态
        self.update_run_state(NodeStateTransitionEvent::StartComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.context.clone();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_run_state(NodeStateTransitionEvent::Stop).await.unwrap();
        // 等待所有任务结束
        self.cancel_task().await?;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_run_state(NodeStateTransitionEvent::StopComplete).await?;
        Ok(())
    }

    async fn update_run_state(&mut self, event: NodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = self.context.read().await.get_node_id().clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, state_manager) = {
            let mut state_manager = self.context.read().await.get_state_machine().clone_box();  // 使用读锁获取当前状态
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        tracing::info!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());
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

                IfElseNodeStateAction::ListenAndHandleMessage => {
                    tracing::info!("{}: 开始监听节点传递的message", node_id);
                    self.listen_message().await?;
                }
                IfElseNodeStateAction::InitReceivedFlag => {
                    tracing::info!("{}: 开始初始化接收标记", node_id);
                    let context = self.get_context();
                    let mut state_guard = context.write().await;
                    if let Some(if_else_node_context) = state_guard.as_any_mut().downcast_mut::<IfElseNodeContext>() {
                        if_else_node_context.init_received_flag().await;
                    }
                }
                IfElseNodeStateAction::InitReceivedValue => {
                    tracing::info!("{}: 开始初始化接收值", node_id);
                    let context = self.get_context();
                    let mut state_guard = context.write().await;
                    if let Some(if_else_node_context) = state_guard.as_any_mut().downcast_mut::<IfElseNodeContext>() {
                        if_else_node_context.init_received_value().await;
                    }
                }
                IfElseNodeStateAction::Evaluate => {
                    tracing::info!("{}: 开始判断条件", node_id);
                    self.evaluate().await;
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


