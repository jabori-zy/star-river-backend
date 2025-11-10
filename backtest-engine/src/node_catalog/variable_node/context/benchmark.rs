use super::VariableNodeContext;
use strategy_core::node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt};
use async_trait::async_trait;
use crate::{node::node_error::BacktestNodeError, strategy::strategy_command::{AddNodeCycleTrackerCmdPayload, AddNodeCycleTrackerCommand}};
use star_river_core::custom_type::NodeId;
use strategy_core::benchmark::node_benchmark::CompletedCycle;

#[async_trait]
impl NodeBenchmarkExt for VariableNodeContext {

    type Error = BacktestNodeError;

    async fn mount_node_cycle_tracker(&self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), Self::Error> {

        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let payload = AddNodeCycleTrackerCmdPayload::new(node_id.clone(), cycle_tracker);
        let command = AddNodeCycleTrackerCommand::new(node_id, resp_tx, payload).into();

        self.send_strategy_command(command).await?;
        resp_rx.await.unwrap();
        Ok(())
    }
}