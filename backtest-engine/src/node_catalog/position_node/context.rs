mod context_util;
mod event_handler;
mod node_handles;
mod config_filter;

use std::sync::Arc;

use async_trait::async_trait;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use star_river_core::custom_type::{NodeId, NodeName};
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    node::{
        context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeMetaDataExt},
        metadata::NodeMetadata,
    },
};
use tokio::sync::{Mutex, broadcast, mpsc};
use virtual_trading::{command::VtsCommand, event::VtsEvent};

use super::{position_node_types::PositionNodeBacktestConfig, state_machine::PositionNodeStateMachine};
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::PositionNodeError, node_event::BacktestNodeEvent},
    strategy::strategy_command::BacktestStrategyCommand,
};

pub type PositionNodeMetadata = NodeMetadata<PositionNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
#[allow(dead_code)]
pub struct PositionNodeContext {
    metadata: PositionNodeMetadata,
    node_config: PositionNodeBacktestConfig,
    database: DatabaseConnection,
    heartbeat: Arc<Mutex<Heartbeat>>,
    vts_command_sender: mpsc::Sender<VtsCommand>,
    pub(crate) vts_event_receiver: broadcast::Receiver<VtsEvent>,
}

impl PositionNodeContext {
    pub fn new(
        metadata: PositionNodeMetadata,
        node_config: PositionNodeBacktestConfig,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        vts_command_sender: mpsc::Sender<VtsCommand>,
        vts_event_receiver: broadcast::Receiver<VtsEvent>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            database,
            heartbeat,
            vts_command_sender,
            vts_event_receiver,
        }
    }
}

impl PositionNodeContext {
    pub fn node_config(&self) -> &PositionNodeBacktestConfig {
        &self.node_config
    }
}

impl NodeMetaDataExt for PositionNodeContext {
    type StateMachine = PositionNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;
    type Error = PositionNodeError;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

#[async_trait]
impl NodeBenchmarkExt for PositionNodeContext {
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
