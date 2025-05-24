pub mod live_strategy_node;
pub mod backtest_strategy_node;

pub mod node_types;
pub mod node_context;
pub mod node_functions;
pub mod node_state_machine;

use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use types::strategy::node_message::NodeMessage;
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::node::node_functions::{LiveNodeFunction, BacktestNodeFunction};
use super::node::node_context::{LiveNodeContextTrait, BacktestNodeContextTrait};
use super::node::node_state_machine::*;
use node_types::*;

#[async_trait]
pub trait LiveNodeTrait: Debug + Send + Sync + 'static {
    // as_any是将类型转换为Any类型
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn LiveNodeTrait>;
    // get方法
    // 获取节点上下文
    fn get_context(&self) -> Arc<RwLock<Box<dyn LiveNodeContextTrait>>>;

    async fn get_from_node_id(&self) -> Vec<String> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_from_node_id().clone()
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
    async fn get_run_state(&self) -> LiveNodeRunState {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_run_state()
    }
    // 获取节点状态机
    async fn get_state_machine(&self) -> Box<dyn LiveNodeStateMachine> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_state_machine()
    }

    async fn get_all_output_handles(&self) -> Vec<NodeOutputHandle> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_all_output_handle().values().cloned().collect()
    }

    async fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_message_sender(handle_id).clone()
    }

    // 获取节点消息接收器
    async fn get_message_receivers(&self) -> Vec<NodeMessageReceiver> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_message_receivers().clone()
    }

    // 获取节点类型
    async fn get_node_type(&self) -> NodeType {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_node_type().clone()
    }



    // 设置节点的出口
    async fn set_output_handle(&mut self) {
        tracing::debug!("{}: 设置节点默认出口", self.get_node_name().await);
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<NodeMessage>(100);

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
            _ => return,
        };

        self.add_output_handle(default_output_handle_id.to_string(), tx).await;
        tracing::debug!("{}: 设置节点默认出口成功: {}", node_name, default_output_handle_id.to_string());
    }

    async fn add_message_receiver(&mut self, receiver: NodeMessageReceiver) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_message_receivers_mut().push(receiver);
    } 
    // 添加出口
    async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<NodeMessage>) {
        let node_output_handle = NodeOutputHandle {
            node_id: self.get_node_id().await,
            output_handle_id: handle_id.clone(),
            message_sender: sender,
            connect_count: 0,
        };

        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_all_output_handle_mut().insert(handle_id, node_output_handle);
    }
    // 增加handle的连接计数
    async fn add_output_handle_connect_count(&mut self, handle_id: String) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_all_output_handle_mut().get_mut(&handle_id).unwrap().connect_count += 1;
    }
    // 添加from_node_id
    async fn add_from_node_id(&mut self, from_node_id: String) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_from_node_id_mut().push(from_node_id);
    }

    // 初始化节点
    async fn init(&mut self) -> Result<(), String>; 
    // 启动节点
    async fn start(&mut self) -> Result<(), String>;
    // 停止节点
    async fn stop(&mut self) -> Result<(), String>;
    // 启用节点事件推送
    async fn enable_node_event_push(&mut self) -> Result<(), String> {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.set_enable_event_publish(true);
        Ok(())
    } 
    // 禁用节点事件推送
    async fn disable_node_event_push(&mut self) -> Result<(), String> {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.set_enable_event_publish(false);
        Ok(())
    } 
    // 监听外部事件
    async fn listen_external_events(&self) -> Result<(), String> {
        let context = self.get_context();
        LiveNodeFunction::listen_external_event(context).await;
        Ok(())
    }
    // 监听节点传递过来的message
    async fn listen_message(&self) -> Result<(), String> {
        let context = self.get_context();
        LiveNodeFunction::listen_message(context).await;
        Ok(())
    }
    // 取消所有异步任务
    async fn cancel_task(&self) -> Result<(), String> {
        let state = self.get_context();
        LiveNodeFunction::cancel_task(state).await;
        Ok(())
    }

    // 更新节点状态
    async fn update_node_state(&mut self, event: LiveNodeStateTransitionEvent) -> Result<(), String>;
}

impl Clone for Box<dyn LiveNodeTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}






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
        context_guard.get_all_output_handle().values().cloned().collect()
    }

    async fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_message_sender(handle_id).clone()
    }

    // 获取节点消息接收器
    async fn get_message_receivers(&self) -> Vec<NodeMessageReceiver> {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_message_receivers().clone()
    }

    // 获取节点类型
    async fn get_node_type(&self) -> NodeType {
        let context = self.get_context();
        let context_guard = context.read().await;
        context_guard.get_node_type().clone()
    }



    // 设置节点的出口
    async fn set_output_handle(&mut self) {
        tracing::debug!("{}: 设置节点默认出口", self.get_node_name().await);
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<NodeMessage>(100);

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
            _ => return,
        };

        self.add_output_handle(default_output_handle_id.to_string(), tx).await;
        tracing::debug!("{}: 设置节点默认出口成功: {}", node_name, default_output_handle_id.to_string());
    }

    async fn add_message_receiver(&mut self, receiver: NodeMessageReceiver) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_message_receivers_mut().push(receiver);
    } 
    // 添加出口
    async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<NodeMessage>) {
        let node_output_handle = NodeOutputHandle {
            node_id: self.get_node_id().await,
            output_handle_id: handle_id.clone(),
            message_sender: sender,
            connect_count: 0,
        };

        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_all_output_handle_mut().insert(handle_id, node_output_handle);
    }
    // 增加handle的连接计数
    async fn add_output_handle_connect_count(&mut self, handle_id: String) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_all_output_handle_mut().get_mut(&handle_id).unwrap().connect_count += 1;
    }
    // 添加from_node_id
    async fn add_from_node_id(&mut self, from_node_id: String) {
        let context = self.get_context();
        let mut context_guard = context.write().await;
        context_guard.get_from_node_id_mut().push(from_node_id);
    }

    // 初始化节点
    async fn init(&mut self) -> Result<(), String>; 
    // 启动节点
    // async fn start(&mut self) -> Result<(), String>;
    // 停止节点
    async fn stop(&mut self) -> Result<(), String>;
    // 启用节点事件推送
    // async fn enable_node_event_push(&mut self) -> Result<(), String> {
    //     let context = self.get_context();
    //     let mut context_guard = context.write().await;
    //     context_guard.set_enable_event_publish(true);
    //     Ok(())
    // } 
    // // 禁用节点事件推送
    // async fn disable_node_event_push(&mut self) -> Result<(), String> {
    //     let context = self.get_context();
    //     let mut context_guard = context.write().await;
    //     context_guard.set_enable_event_publish(false);
    //     Ok(())
    // } 
    // 监听外部事件
    async fn listen_external_events(&self) -> Result<(), String> {
        let context = self.get_context();
        BacktestNodeFunction::listen_external_event(context).await;
        Ok(())
    }
    // 监听节点传递过来的message
    async fn listen_message(&self) -> Result<(), String> {
        let context = self.get_context();
        BacktestNodeFunction::listen_message(context).await;
        Ok(())
    }
    // 取消所有异步任务
    async fn cancel_task(&self) -> Result<(), String> {
        let state = self.get_context();
        BacktestNodeFunction::cancel_task(state).await;
        Ok(())
    }

    // 更新节点状态
    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), String>;
}

impl Clone for Box<dyn BacktestNodeTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}



