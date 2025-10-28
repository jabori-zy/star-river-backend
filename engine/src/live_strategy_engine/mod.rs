

use crate::{Engine, EngineContext};
use event_center::communication::engine::EngineCommand;
use event_center::event::Event;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::RwLock;


// #[async_trait]
// pub trait LiveNodeTrait: Debug + Send + Sync + 'static {
//     // as_any是将类型转换为Any类型
//     fn as_any(&self) -> &dyn Any;

//     fn as_any_mut(&mut self) -> &mut dyn Any;

//     fn clone_box(&self) -> Box<dyn LiveNodeTrait>;
//     // get方法
//     // 获取节点上下文
//     fn get_context(&self) -> Arc<RwLock<Box<dyn LiveNodeContextTrait>>>;

//     async fn get_from_node_id(&self) -> Vec<String> {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_from_node_id().clone()
//     }

//     // 获取节点id
//     async fn get_node_id(&self) -> String {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_node_id().clone()
//     }
//     // 获取节点名称
//     async fn get_node_name(&self) -> String {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_node_name().clone()
//     }
//     // 获取节点运行状态
//     async fn get_run_state(&self) -> LiveNodeRunState {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_run_state()
//     }
//     // 获取节点状态机
//     async fn get_state_machine(&self) -> Box<dyn LiveNodeStateMachine> {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_state_machine()
//     }

//     async fn get_all_output_handles(&self) -> Vec<NodeOutputHandle> {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_all_output_handle().values().cloned().collect()
//     }

//     // 获取节点消息接收器
//     async fn get_message_receivers(&self) -> Vec<NodeInputHandle> {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_message_receivers().clone()
//     }

//     // 获取节点类型
//     async fn get_node_type(&self) -> NodeType {
//         let context = self.get_context();
//         let context_guard = context.read().await;
//         context_guard.get_node_type().clone()
//     }

//     // 设置节点的出口
//     async fn set_output_handle(&mut self) {
//         tracing::debug!("{}: 设置节点默认出口", self.get_node_name().await);
//         let node_name = self.get_node_name().await;
//         let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);

//         let node_type = self.get_node_type().await;

//         if node_type == NodeType::GetVariableNode {
//             tracing::debug!("{}: 变量节点没有默认出口", node_name);
//             return;
//         }

//         let default_output_handle_id = match node_type {
//             NodeType::StartNode => DefaultOutputHandleId::StartNodeOutput,
//             NodeType::KlineNode => DefaultOutputHandleId::KlineNodeOutput,
//             NodeType::IndicatorNode => DefaultOutputHandleId::IndicatorNodeOutput,
//             NodeType::IfElseNode => DefaultOutputHandleId::IfElseNodeElseOutput,
//             NodeType::OrderNode => DefaultOutputHandleId::OrderNodeOutput,
//             NodeType::PositionNode => DefaultOutputHandleId::PositionNodeUpdateOutput,
//             _ => return,
//         };

//         self.add_output_handle(default_output_handle_id.to_string(), tx).await;
//         tracing::debug!("{}: 设置节点默认出口成功: {}", node_name, default_output_handle_id.to_string());
//     }

//     async fn add_message_receiver(&mut self, receiver: NodeInputHandle) {
//         let context = self.get_context();
//         let mut context_guard = context.write().await;
//         context_guard.get_message_receivers_mut().push(receiver);
//     }
//     // 添加出口
//     async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<BacktestNodeEvent>) {
//         let node_id = self.get_node_id().await;
//         let node_output_handle = NodeOutputHandle::new(node_id, handle_id.clone(), sender);

//         let context = self.get_context();
//         let mut context_guard = context.write().await;
//         context_guard.get_all_output_handle_mut().insert(handle_id, node_output_handle);
//     }

//     // 添加from_node_id
//     async fn add_from_node_id(&mut self, from_node_id: String) {
//         let context = self.get_context();
//         let mut context_guard = context.write().await;
//         context_guard.get_from_node_id_mut().push(from_node_id);
//     }

//     // 初始化节点
//     async fn init(&mut self) -> Result<(), String>;
//     // 启动节点
//     async fn start(&mut self) -> Result<(), String>;
//     // 停止节点
//     async fn stop(&mut self) -> Result<(), String>;
//     // 启用节点事件推送
//     async fn enable_node_event_push(&mut self) -> Result<(), String> {
//         let context = self.get_context();
//         let mut context_guard = context.write().await;
//         context_guard.set_enable_event_publish(true);
//         Ok(())
//     }
//     // 禁用节点事件推送
//     async fn disable_node_event_push(&mut self) -> Result<(), String> {
//         let context = self.get_context();
//         let mut context_guard = context.write().await;
//         context_guard.set_enable_event_publish(false);
//         Ok(())
//     }
//     // 监听外部事件
//     async fn listen_external_events(&self) -> Result<(), String> {
//         let context = self.get_context();
//         LiveNodeFunction::listen_external_event(context).await;
//         Ok(())
//     }
//     // 监听节点传递过来的message
//     async fn listen_message(&self) -> Result<(), String> {
//         let context = self.get_context();
//         LiveNodeFunction::listen_message(context).await;
//         Ok(())
//     }
//     // 取消所有异步任务
//     async fn cancel_task(&self) -> Result<(), String> {
//         let state = self.get_context();
//         LiveNodeFunction::cancel_task(state).await;
//         Ok(())
//     }

//     // 更新节点状态
//     async fn update_node_state(&mut self, event: LiveNodeStateTransitionEvent) -> Result<(), String>;
// }

// impl Clone for Box<dyn LiveNodeTrait> {
//     fn clone(&self) -> Self {
//         self.clone_box()
//     }
// }

#[derive(Debug, Clone)]
pub struct LiveStrategyEngine {
    // 简化的上下文，付费版本可以有更复杂的实现
    context: Arc<RwLock<Box<dyn EngineContext>>>,
}

impl LiveStrategyEngine {
    pub fn new() -> Self {
        // 创建一个空的上下文实现
        let context = Arc::new(RwLock::new(Box::new(DummyEngineContext) as Box<dyn EngineContext>));
        Self { context }
    }
}

impl Engine for LiveStrategyEngine {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Engine> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn EngineContext>>> {
        self.context.clone()
    }
}

// 简单的空实现，付费版本可以有完整实现
#[derive(Debug)]
pub struct DummyEngineContext;

#[async_trait::async_trait]
impl EngineContext for DummyEngineContext {
    fn get_engine_name(&self) -> star_river_core::engine::EngineName {
        star_river_core::engine::EngineName::LiveStrategyEngine
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(DummyEngineContext)
    }

    async fn handle_event(&mut self, _event: Event) {
        // 空实现，付费版本可以有实际逻辑
    }

    async fn handle_command(&mut self, _command: EngineCommand) {
        // 空实现，付费版本可以有实际逻辑
    }
}