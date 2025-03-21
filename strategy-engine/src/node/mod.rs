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
use std::error::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum NodeRunState {
    Created,
    Ready,
    Running,
    Stopping,
    Stopped,
}

#[async_trait]
pub trait NodeTrait: Debug + Send + Sync  {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn NodeTrait>;
    async fn get_node_sender(&self, handle_id: String) -> NodeSender;
    async fn get_default_node_sender(&self) -> NodeSender;
    async fn get_node_name(&self) -> String;
    async fn get_node_id(&self) -> String;
    fn add_message_receiver(&mut self, receiver: NodeReceiver); // 添加接收者
    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender); // 添加出口
    async fn add_node_output_handle_connect_count(&mut self, handle_id: String);// 增加handle的连接计数
    fn add_from_node_id(&mut self, from_node_id: String); // 添加from_node_id
    async fn enable_node_event_publish(&mut self); // 启用节点事件发布
    async fn disable_node_event_publish(&mut self); // 禁用节点事件发布
    async fn init(&mut self) -> Result<NodeRunState, Box<dyn Error>>;
    async fn get_node_run_state(&self) -> NodeRunState;
}

impl Clone for Box<dyn NodeTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
