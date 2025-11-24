use async_trait::async_trait;
use event_center::event::Event;
use strategy_core::node::context_trait::{NodeEventHandlerExt, NodeInfoExt};

use super::StartNodeContext;
use crate::node::{
    node_command::{
        BacktestNodeCommand, GetStartNodeConfigRespPayload, GetStartNodeConfigResponse, NodeResetRespPayload, NodeResetResponse,
    },
    node_error::StartNodeError,
};
#[async_trait]
impl NodeEventHandlerExt for StartNodeContext {
    type EngineEvent = Event;
    type Error = StartNodeError;

    /// 处理引擎事件
    async fn handle_engine_event(&mut self, _event: Self::EngineEvent) -> Result<(), StartNodeError> {
        Ok(())
    }

    /// 处理节点事件
    async fn handle_source_node_event(&mut self, _node_event: Self::NodeEvent) -> Result<(), StartNodeError> {
        Ok(())
    }

    async fn handle_command(&mut self, node_command: Self::NodeCommand) -> Result<(), StartNodeError> {
        match node_command {
            BacktestNodeCommand::GetStartNodeConfig(cmd) => {
                let start_node_config = self.node_config.read().await.clone();

                let payload = GetStartNodeConfigRespPayload::new(start_node_config);
                let response = GetStartNodeConfigResponse::success(self.node_id().clone(), payload);
                cmd.respond(response);
                Ok(())
            }
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == cmd.node_id() {
                    let payload = NodeResetRespPayload {};
                    let response = NodeResetResponse::success(self.node_id().clone(), payload);
                    cmd.respond(response);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            _ => Ok(()),
        }
    }
}
