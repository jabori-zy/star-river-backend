pub mod futures_order_node;
pub mod if_else_node;
pub mod indicator_node;
pub mod kline_node;
pub mod node_message;
pub mod node_utils;
pub mod position_node;
pub mod start_node;
pub mod variable_node;
pub mod context_accessor;
// pub mod benchmark;

pub mod node_context;
pub mod node_functions;
pub mod node_state_machine;
pub mod node_handles;

pub use context_accessor::BacktestNodeContextAccessor;
pub use node_context::BacktestNodeContextTrait;
pub use futures_order_node::FuturesOrderNode;
pub use if_else_node::IfElseNode;
pub use indicator_node::IndicatorNode;
pub use kline_node::KlineNode;
pub use position_node::PositionManagementNode;
pub use start_node::StartNode;
pub use variable_node::VariableNode;

pub mod context {
    pub use super::futures_order_node::futures_order_node_context::FuturesOrderNodeContext;
    pub use super::if_else_node::if_else_node_context::IfElseNodeContext;
    pub use super::indicator_node::indicator_node_context::IndicatorNodeContext;
    pub use super::kline_node::kline_node_context::KlineNodeContext;
    pub use super::position_node::position_node_context::PositionNodeContext;
    pub use super::start_node::start_node_context::StartNodeContext;
    pub use super::variable_node::variable_node_context::VariableNodeContext;
}
use super::node::node_functions::BacktestNodeFunction;
use super::node::node_state_machine::*;
use async_trait::async_trait;
use node_handles::*;
use star_river_core::error::engine_error::strategy_engine_error::node_error::BacktestStrategyNodeError;
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;



#[async_trait]
pub trait BacktestNodeTrait: Debug + Send + Sync + 'static {
    // as_any是将类型转换为Any类型
    fn as_any(&self) -> &dyn Any;

    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn clone_box(&self) -> Box<dyn BacktestNodeTrait>;

    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>;

    // 设置节点的出口
    async fn set_output_handle(&mut self) -> Result<(), BacktestStrategyNodeError>;

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