use async_trait::async_trait;
use snafu::ResultExt;
use star_river_core::custom_type::NodeId;
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    error::node_error::StrategyCommandRespRecvFailedSnafu,
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt},
};

use super::VariableNodeContext;
use crate::{
    node::node_error::BacktestNodeError,
    strategy::strategy_command::{AddNodeCycleTrackerCmdPayload, AddNodeCycleTrackerCommand},
};

#[async_trait]
impl NodeBenchmarkExt for VariableNodeContext {
    type Error = BacktestNodeError;

    async fn mount_node_cycle_tracker(&self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), Self::Error> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let payload = AddNodeCycleTrackerCmdPayload::new(node_id.clone(), cycle_tracker);
        let command = AddNodeCycleTrackerCommand::new(node_id.clone(), resp_tx, payload).into();

        self.send_strategy_command(command).await?;
        resp_rx.await.context(StrategyCommandRespRecvFailedSnafu {
            node_id: node_id.to_string(),
        })?;
        Ok(())
    }
}
