// ============================================================================
// 外部 crate 导入
// ============================================================================

use async_trait::async_trait;
use event_center::event::Event;
use strategy_core::node::context_trait::{NodeEventHandlerExt, NodeIdentityExt};

// ============================================================================
// 当前模块内部导入（相对路径）
// ============================================================================
use super::StartNodeContext;
// ============================================================================
// 当前 crate 内部导入（使用绝对路径）
// ============================================================================
use crate::node::node_command::{
    BacktestNodeCommand, GetStartNodeConfigRespPayload, GetStartNodeConfigResponse, NodeResetRespPayload, NodeResetResponse,
};

#[async_trait]
impl NodeEventHandlerExt for StartNodeContext {
    type EngineEvent = Event;

    /// 处理引擎事件
    async fn handle_engine_event(&mut self, event: Self::EngineEvent) {
        tracing::info!("[{}] received engine event: {:?}", self.node_name(), event);
    }

    /// 处理节点事件
    async fn handle_node_event(&mut self, node_event: Self::NodeEvent) {
        tracing::info!("[{}] received node event: {:?}", self.node_name(), node_event);
    }

    async fn handle_node_command(&mut self, node_command: Self::NodeCommand) {
        match node_command {
            BacktestNodeCommand::GetStartNodeConfig(cmd) => {
                let start_node_config = self.node_config.read().await.clone();

                let payload = GetStartNodeConfigRespPayload::new(start_node_config);
                let response = GetStartNodeConfigResponse::success(self.node_id().clone(), payload);
                cmd.respond(response);
            }
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let payload = NodeResetRespPayload {};
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                }
            }
        }
    }
}
