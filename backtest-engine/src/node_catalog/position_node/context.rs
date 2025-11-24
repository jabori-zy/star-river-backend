mod context_util;
mod event_handler;
mod node_handles;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
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
use tokio::sync::Mutex;

use super::{position_node_types::PositionNodeBacktestConfig, state_machine::PositionNodeStateMachine};
use crate::{
    node::{node_command::BacktestNodeCommand, node_event::BacktestNodeEvent},
    strategy::strategy_command::BacktestStrategyCommand,
    virtual_trading_system::BacktestVts,
};

pub type PositionNodeMetadata = NodeMetadata<PositionNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct PositionNodeContext {
    metadata: PositionNodeMetadata,
    node_config: PositionNodeBacktestConfig,
    database: DatabaseConnection,
    heartbeat: Arc<Mutex<Heartbeat>>,
    virtual_trading_system: Arc<Mutex<BacktestVts>>,
    current_time_watch_rx: tokio::sync::watch::Receiver<DateTime<Utc>>,
}

impl PositionNodeContext {
    pub fn new(
        metadata: PositionNodeMetadata,
        node_config: PositionNodeBacktestConfig,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        virtual_trading_system: Arc<Mutex<BacktestVts>>,
        current_time_watch_rx: tokio::sync::watch::Receiver<DateTime<Utc>>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            database,
            heartbeat,
            virtual_trading_system,
            current_time_watch_rx,
        }
    }
}

impl PositionNodeContext {
    pub fn current_time(&self) -> DateTime<Utc> {
        *self.current_time_watch_rx.borrow()
    }

    pub fn node_config(&self) -> &PositionNodeBacktestConfig {
        &self.node_config
    }
}

impl NodeMetaDataExt for PositionNodeContext {
    type StateMachine = PositionNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

#[async_trait]
impl NodeBenchmarkExt for PositionNodeContext {
    type Error = crate::node::node_error::BacktestNodeError;

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
