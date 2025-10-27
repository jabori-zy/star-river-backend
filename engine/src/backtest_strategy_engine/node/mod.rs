pub mod futures_order_node;
pub mod if_else_node;
pub mod indicator_node;
pub mod kline_node;
pub mod node_message;
pub mod node_utils;
pub mod position_management_node;
pub mod start_node;
pub mod variable_node;
// pub mod benchmark;

pub mod node_context;
pub mod node_functions;
pub mod node_state_machine;
pub mod node_handles;

pub use futures_order_node::FuturesOrderNode;
pub use if_else_node::IfElseNode;
pub use indicator_node::IndicatorNode;
pub use kline_node::KlineNode;
pub use position_management_node::PositionManagementNode;
pub use start_node::StartNode;
pub use variable_node::VariableNode;

pub mod context {
    pub use super::futures_order_node::futures_order_node_context::FuturesOrderNodeContext;
    pub use super::if_else_node::if_else_node_context::IfElseNodeContext;
    pub use super::indicator_node::indicator_node_context::IndicatorNodeContext;
    pub use super::kline_node::kline_node_context::KlineNodeContext;
    pub use super::position_management_node::position_management_node_context::PositionNodeContext;
    pub use super::start_node::start_node_context::StartNodeContext;
    pub use super::variable_node::variable_node_context::VariableNodeContext;
}

use super::node::node_context::{BacktestNodeContextTrait, LiveNodeContextTrait};
use super::node::node_functions::{BacktestNodeFunction, LiveNodeFunction};
use super::node::node_state_machine::*;
use async_trait::async_trait;
use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::strategy_event::NodeStateLogEvent;
use node_handles::*;
use star_river_core::error::StarRiverErrorTrait;
use star_river_core::error::engine_error::strategy_engine_error::node_error::BacktestStrategyNodeError;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast;

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

#[async_trait]
pub trait BacktestNodeTrait: Debug + Send + Sync + 'static {
    // as_any是将类型转换为Any类型
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn BacktestNodeTrait>;
    // get方法
    // 获取节点上下文
    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>;

    async fn get_from_node_id(&self) -> Vec<String> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_from_node_id().clone()
    }

    async fn get_strategy_id(&self) -> i32 {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_strategy_id().clone()
    }

    // 获取节点id
    async fn get_node_id(&self) -> String {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_node_id().clone()
    }
    // 获取节点名称
    async fn get_node_name(&self) -> String {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_node_name().clone()
    }
    // 获取节点运行状态
    async fn get_run_state(&self) -> BacktestNodeRunState {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_run_state()
    }
    // 获取节点状态机
    async fn get_state_machine(&self) -> Box<dyn BacktestNodeStateMachine> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_state_machine()
    }

    async fn get_all_output_handles(&self) -> Vec<NodeOutputHandle> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_all_output_handles().values().cloned().collect()
    }

    // 获取节点消息接收器
    async fn get_node_event_receivers(&self) -> Vec<NodeInputHandle> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_all_input_handles().clone()
    }

    // 获取节点类型
    async fn get_node_type(&self) -> NodeType {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_node_type().clone()
    }

    async fn set_is_leaf_node(&mut self, is_leaf_node: bool) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.set_is_leaf_node(is_leaf_node);
    }

    async fn is_leaf_node(&self) -> bool {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.is_leaf_node()
    }

    // 获取节点输出句柄
    async fn get_output_handle(&self, handle_id: &String) -> NodeOutputHandle {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_output_handle(handle_id).clone()
    }


    async fn subscribe_output_handle(&self, subscriber_id: String, handle_id: &String) -> broadcast::Receiver<BacktestNodeEvent> {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_output_handle_mut(handle_id).subscribe(subscriber_id)
    }

    // 获取向strategy发送的出口句柄
    async fn get_strategy_output_handle(&self) -> NodeOutputHandle {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_strategy_output_handle().clone()
    }

    // 设置节点的出口
    async fn set_output_handle(&mut self) {
        tracing::debug!("{}: 设置节点默认出口", self.get_node_name().await);
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);

        let node_type = self.get_node_type().await;

        if node_type == NodeType::GetVariableNode {
            tracing::debug!("{}: 变量节点没有默认出口", node_name);
            return;
        }

        let default_output_handle_id = match node_type {
            NodeType::StartNode => DefaultOutputHandleId::StartNodeOutput,
            NodeType::KlineNode => DefaultOutputHandleId::KlineNodeOutput,
            NodeType::IndicatorNode => DefaultOutputHandleId::IndicatorNodeOutput,
            NodeType::IfElseNode => DefaultOutputHandleId::IfElseNodeElseOutput,
            NodeType::OrderNode => DefaultOutputHandleId::OrderNodeOutput,
            NodeType::PositionNode => DefaultOutputHandleId::PositionNodeUpdateOutput,
            NodeType::GetVariableNode => DefaultOutputHandleId::GetVariableNodeOutput,
            _ => return,
        };

        self.add_output_handle(default_output_handle_id.to_string(), tx).await;
        tracing::debug!("{}: 设置节点默认出口成功: {}", node_name, default_output_handle_id.to_string());
    }

    async fn add_input_handle(&mut self, input_handle: NodeInputHandle) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_all_input_handles_mut().push(input_handle);
    }
    // 添加出口
    async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<BacktestNodeEvent>) {
        let node_id = self.get_node_id().await;
        let node_output_handle = NodeOutputHandle::new(node_id, handle_id.clone(), sender);

        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_all_output_handle_mut().insert(handle_id, node_output_handle);
    }

    // 添加from_node_id
    async fn add_from_node_id(&mut self, from_node_id: String) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_from_node_id_mut().push(from_node_id);
    }

    // 初始化节点
    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError>;

    // 停止节点
    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError>;

    // 监听外部事件
    async fn listen_external_events(&self) {
        let context = self.get_context();
        BacktestNodeFunction::listen_external_event(context).await;
    }

    // 监听节点传递过来的message
    async fn listen_node_events(&self) {
        let context = self.get_context();
        BacktestNodeFunction::listen_node_events(context).await;
    }

    // 监听策略命令
    async fn listen_strategy_command(&self) {
        let context = self.get_context();
        BacktestNodeFunction::listen_strategy_command(context).await;
    }

    // 取消所有异步任务
    async fn cancel_task(&self) {
        let context = self.get_context();
        BacktestNodeFunction::cancel_task(context).await;
    }

    // 更新节点状态
    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), BacktestStrategyNodeError>;
}

impl Clone for Box<dyn BacktestNodeTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}