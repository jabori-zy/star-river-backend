use async_trait::async_trait;
use star_river_core::custom_type::{NodeId, NodeName};
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt},
};

use super::VariableNodeContext;

#[async_trait]
impl NodeBenchmarkExt for VariableNodeContext {
    async fn mount_node_cycle_tracker(
        &self,
        node_id: NodeId,
        node_name: NodeName,
        cycle_tracker: CompletedCycle,
    ) -> Result<(), Self::Error> {
        crate::node::node_utils::NodeUtils::mount_node_cycle_tracker(node_id, node_name, cycle_tracker, self.strategy_command_sender())
            .await?;
        Ok(())
    }
}
