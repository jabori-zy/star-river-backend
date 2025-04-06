pub mod indicator_node_state_machine;
pub mod indicator_node_context;

use types::indicator::Indicators;
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use types::market::{Exchange, KlineInterval};
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::node::NodeTrait;
use crate::NodeType;
use crate::NodeRunState;
use crate::node::NodeStateTransitionEvent;
use crate::node::indicator_node::indicator_node_state_machine::IndicatorNodeStateManager;
use crate::node::indicator_node::indicator_node_state_machine::IndicatorNodeStateAction;
use std::time::Duration;
use crate::node::indicator_node::indicator_node_context::IndicatorNodeState;
use event_center::EventPublisher;
use event_center::Event;
use crate::node::node_context::BaseNodeContext;
use crate::node::node_context::Context;





// 指标节点
#[derive(Debug, Clone)]
pub struct IndicatorNode {
    pub context: Arc<RwLock<Box<dyn Context>>>,
    
}




impl IndicatorNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        indicator: Indicators, 
        event_publisher: EventPublisher, 
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let base_context = BaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::IndicatorNode,
            event_publisher,
            vec![response_event_receiver],
            Box::new(IndicatorNodeStateManager::new(NodeRunState::Created, node_id, node_name)),
        );

        Self {
            context: Arc::new(RwLock::new(Box::new(IndicatorNodeState {
                base_context,
                exchange,
                symbol,
                interval,
                indicator,
                current_batch_id: None,
                request_id: None,
            }))),
            
        }
    }
    
    
}

#[async_trait]
impl NodeTrait for IndicatorNode {

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn Context>>> {
        self.context.clone()
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
        tracing::info!("{}: 开始启动", self.get_node_id().await);
        self.update_run_state(NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        self.update_run_state(NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_run_state(NodeStateTransitionEvent::Stop).await.unwrap();

        // 等待所有任务结束
        self.cancel_task().await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_run_state(NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn update_run_state(&mut self, event: NodeStateTransitionEvent) -> Result<(), String> {
        let node_id = self.get_node_id().await;
        
        // 获取状态管理器并执行转换
        let (transition_result, state_machine) = {
            let mut state_machine = self.get_state_machine().await;
            let transition_result = state_machine.transition(event)?;
            (transition_result, state_machine)
        };

        tracing::info!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(indicator_node_state_action) = action.as_any().downcast_ref::<IndicatorNodeStateAction>() {
                match indicator_node_state_action {
                    IndicatorNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    IndicatorNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    IndicatorNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    IndicatorNodeStateAction::ListenAndHandleMessage => {
                        tracing::info!("{}: 开始监听节点传递的message", node_id);
                        self.listen_message().await?;
                    }
                    _ => {}
                }
                // 所有动作执行完毕后更新节点最新的状态
                {
                    self.context.write().await.set_state_machine(state_machine.clone_box());
                }
            }
        }
                    
        Ok(())
    }
}