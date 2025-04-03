use std::sync::Arc;
use tokio::sync::RwLock;
use crate::NodeType;
use crate::NodeOutputHandle;
use std::collections::HashMap;
use crate::node::NodeStateTransitionEvent;
use tokio::time::Duration;
use async_trait::async_trait;
use crate::NodeRunState;


#[async_trait]
pub trait BaseNodeAction: Send + Sync {
    type State: Send + Sync + 'static;

    // 获取节点状态
    fn get_state(&self) -> Arc<RwLock<Self::State>>;

    // 获取节点id
    async fn get_node_id(&self) -> String;

    // 获取节点名称
    async fn get_node_name(&self) -> String;

    // 获取节点输出句柄
    async fn get_node_output_handle(&self) -> HashMap<String, NodeOutputHandle>;

    /// 获取节点运行状态
    async fn get_node_run_state(&self) -> NodeRunState;

    // 更新节点状态
    async fn update_run_state(&self, event: NodeStateTransitionEvent) -> Result<(), String>;
    
    // 初始化节点
    async fn init(&mut self) -> Result<(), String>;

    // 启动节点
    async fn start(&mut self) -> Result<(), String>;

    // 停止节点
    async fn stop(&mut self) -> Result<(), String>;

    // 取消任务
    async fn cancel_task(&mut self) -> Result<(), String>;


}

