// pub mod live_data_node;
// pub mod indicator_node;
// pub mod if_else_node;
pub mod start_node;
// pub mod order_node;
pub mod node_context;
pub mod node_functions;
pub mod state_machine;


use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use crate::NodeMessageReceiver;
use tokio::sync::broadcast;
use crate::NodeMessage;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::node::node_functions::NodeFunction;
use crate::node::node_context::Context;
use crate::node::node_context::BaseNodeContext;
use crate::node::state_machine::NodeStateMachine;


#[async_trait]
pub trait NodeTrait: Debug + Send + Sync + 'static {
    // as_any是将类型转换为Any类型
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn NodeTrait>;
    // get方法
    // 获取节点状态
    async fn get_context(&self) -> Arc<RwLock<Box<dyn Context>>>;

    
    // 获取节点id
    async fn get_node_id(&self) -> String {
        let context = self.get_context().await;
        let context_guard = context.read().await;
        context_guard.get_node_id().clone()
    }
    // 获取节点名称
    async fn get_node_name(&self) -> String {
        let context = self.get_context().await;
        let context_guard = context.read().await;
        context_guard.get_node_name().clone()
    }
    // 获取节点运行状态
    async fn get_run_state(&self) -> NodeRunState {
        let context = self.get_context().await;
        let context_guard = context.read().await;
        context_guard.get_run_state()
    }
    // 获取节点状态机
    async fn get_state_machine(&self) -> Box<dyn NodeStateMachine> {
        let context = self.get_context().await;
        let context_guard = context.read().await;
        context_guard.get_state_machine().clone_box()
    }

    async fn add_message_receiver(&mut self, receiver: NodeMessageReceiver); 
    // 添加出口
    async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<NodeMessage>); // 添加出口
    // 增加handle的连接计数
    async fn add_output_handle_connect_count(&mut self, handle_id: String);
    // 添加from_node_id
    async fn add_from_node_id(&mut self, from_node_id: String); 

    // 初始化节点
    async fn init(&mut self) -> Result<(), String>; 
    // 启动节点
    async fn start(&mut self) -> Result<(), String> {
        Ok(())
    } 
    // 停止节点
    async fn stop(&mut self) -> Result<(), String> {
        Ok(())
    } 
    // 启用节点事件推送
    async fn enable_node_event_push(&mut self); 
    // 禁用节点事件推送
    async fn disable_node_event_push(&mut self); 
    // 监听外部事件
    async fn listen_external_events(&self) -> Result<(), String> {
        let state = self.get_context().await;
        NodeFunction::listen_external_event(state).await;
        Ok(())
    }
    // 监听节点传递过来的message
    async fn listen_message(&self) -> Result<(), String> {
        Ok(())
    }
    // 取消所有异步任务
    async fn cancel_task(&self) -> Result<(), String> {
        Ok(())
    }
}

impl Clone for Box<dyn NodeTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}




// 节点运行状态
#[derive(Debug, Clone, PartialEq)]
pub enum NodeRunState {
    Created,        // 节点已创建但未初始化
    Initializing,   // 节点正在初始化
    Ready,          // 节点已初始化，准备好但未运行
    Starting,       // 节点正在启动
    Running,        // 节点正在运行
    Stopping,       // 节点正在停止
    Stopped,        // 节点已停止
    Failed,         // 节点发生错误
}


// 状态转换事件
#[derive(Debug)]
pub enum NodeStateTransitionEvent {
    Initialize,     // 初始化开始
    InitializeComplete,  // 初始化完成 -> 进入Ready状态
    Start,          // 启动节点
    StartComplete,  // 启动完成 -> 进入Running状态
    Stop,           // 停止节点
    StopComplete,   // 停止完成 -> 进入Stopped状态
    Fail(String),   // 节点失败，带有错误信息
}