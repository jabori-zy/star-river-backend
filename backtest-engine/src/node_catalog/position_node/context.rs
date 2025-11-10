mod benchmark;
mod event_handler;
mod node_handles;

use std::sync::Arc;

use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use strategy_core::node::{context_trait::NodeMetaDataExt, metadata::NodeMetadata};
use tokio::sync::Mutex;
use virtual_trading::VirtualTradingSystem;

use super::{position_node_types::PositionNodeBacktestConfig, state_machine::PositionNodeStateMachine};
use crate::{
    node::{node_command::BacktestNodeCommand, node_event::BacktestNodeEvent},
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
};

pub type PositionNodeMetadata = NodeMetadata<PositionNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct PositionNodeContext {
    metadata: PositionNodeMetadata,
    node_config: PositionNodeBacktestConfig,
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    database: DatabaseConnection,
    heartbeat: Arc<Mutex<Heartbeat>>,
    virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
}

impl PositionNodeContext {
    pub fn new(
        metadata: PositionNodeMetadata,
        node_config: PositionNodeBacktestConfig,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            play_index_watch_rx,
            database,
            heartbeat,
            virtual_trading_system,
        }
    }
}

impl PositionNodeContext {
    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
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
