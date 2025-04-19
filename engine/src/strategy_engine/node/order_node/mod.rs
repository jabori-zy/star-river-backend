
mod order_node_state_machine;
mod order_node_context;
pub mod order_node_types;


use types::market::Exchange;
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use std::sync::Arc;
use tokio::sync::RwLock;
use event_center::EventPublisher;
use super::order_node::order_node_state_machine::{OrderNodeStateMachine, OrderNodeStateAction};
use super::{NodeTrait,NodeStateTransitionEvent,NodeType};
use std::time::Duration;
use super::node_context::{NodeContext,BaseNodeContext};
use super::order_node::order_node_context::OrderNodeContext;
use super::order_node::order_node_types::OrderConfig;





#[derive(Debug, Clone)]
pub struct OrderNode {
    pub context: Arc<RwLock<Box<dyn NodeContext>>>,

}

impl OrderNode {
    pub fn new(
        strategy_id: i64,
        node_id: String,
        node_name: String,
        account_id: i32,
        exchange: Exchange,
        symbol: String,
        order_config: OrderConfig,
        event_publisher: EventPublisher,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let base_context = BaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::OrderNode,
            event_publisher,
            vec![response_event_receiver],
            Box::new(OrderNodeStateMachine::new(node_id, node_name)),
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(OrderNodeContext {
                base_context,
                account_id,
                exchange,
                symbol,
                order_config,
                request_id: None,
            }))),
        }
    }
    
}



#[async_trait]
impl NodeTrait for OrderNode {
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

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.get_node_name().await);
        tracing::info!("{}: 开始初始化", self.get_node_name().await);
        // 开始初始化 created -> Initialize
        self.update_run_state(NodeStateTransitionEvent::Initialize).await.unwrap();

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!("{:?}: 初始化完成", self.get_state_machine().await.current_state());
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

        tracing::debug!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());

        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(order_node_state_action) = action.as_any().downcast_ref::<OrderNodeStateAction>() {
                match order_node_state_action {
                    OrderNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    OrderNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    OrderNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    OrderNodeStateAction::ListenAndHandleMessage => {
                        tracing::info!("{}: 开始监听节点消息", node_id);
                        self.listen_message().await?;
                    }
                    OrderNodeStateAction::LogError(error) => {
                        tracing::error!("{}: 发生错误: {}", node_id, error);
                    }
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


