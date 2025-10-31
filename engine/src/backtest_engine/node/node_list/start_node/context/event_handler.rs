// ============================================================================
// 外部 crate 导入
// ============================================================================

use async_trait::async_trait;
use event_center::communication::{
    Command,
    backtest_strategy::{BacktestNodeCommand, GetStartNodeConfigRespPayload, GetStartNodeConfigResponse, NodeResetResponse},
};

// ============================================================================
// 当前 crate 内部导入（使用绝对路径）
// ============================================================================

use crate::backtest_engine::node::node_context_trait::{NodeEventHandler, NodeIdentity};

// ============================================================================
// 当前模块内部导入（相对路径）
// ============================================================================

use super::{StartNodeAction, StartNodeContext};

#[async_trait]
impl NodeEventHandler<StartNodeAction> for StartNodeContext {
    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        match node_command {
            BacktestNodeCommand::GetStartNodeConfig(cmd) => {
                let start_node_config = self.node_config.read().await.clone();

                let payload = GetStartNodeConfigRespPayload::new(start_node_config);
                let response = GetStartNodeConfigResponse::success(self.node_id().clone(), Some(payload));
                cmd.respond(response);
            }
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.node_id() == &cmd.node_id() {
                    let response = NodeResetResponse::success(self.node_id().clone(), None);
                    cmd.respond(response);
                }
            }
        }
    }
}
