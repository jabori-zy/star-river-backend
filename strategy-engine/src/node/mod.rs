pub mod live_data_node;
pub mod indicator_node;
pub mod if_else_node;
pub mod start_node;
pub mod buy_node;


use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use crate::NodeSender;
use crate::NodeReceiver;

#[async_trait]
pub trait NodeTrait: Debug + Send + Sync  {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn NodeTrait>;
    async fn get_node_sender(&self, handle_id: String) -> NodeSender;
    async fn get_default_node_sender(&self) -> NodeSender;
    async fn get_node_name(&self) -> String;
    async fn get_node_id(&self) -> String;
    async fn add_message_receiver(&mut self, receiver: NodeReceiver); // 添加接收者
    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender); // 添加出口
    async fn add_node_output_handle_connect_count(&mut self, handle_id: String);// 增加handle的连接计数
    async fn add_from_node_id(&mut self, from_node_id: String); // 添加from_node_id

    async fn get_node_run_state(&self) -> NodeRunState; // 获取节点运行状态
    async fn init(&mut self) -> Result<(), String>; // 初始化节点
    async fn start(&mut self) -> Result<(), String> {
        Ok(())
    } // 启动节点
    async fn stop(&mut self) -> Result<(), String> {
        Ok(())
    } // 停止节点
    async fn enable_node_event_push(&mut self); // 启用节点事件推送
    async fn disable_node_event_push(&mut self); // 禁用节点事件推送
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