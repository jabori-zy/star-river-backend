mod benchmark;
mod node_handles;
mod event_handler;

use strategy_core::node::metadata::NodeMetadata;
use crate::node::node_event::BacktestNodeEvent;
use crate::node::node_command::BacktestNodeCommand;
use crate::strategy::strategy_command::BacktestStrategyCommand;
use super::state_machine::PositionNodeStateMachine;
use super::position_node_types::PositionNodeBacktestConfig;
use crate::strategy::PlayIndex;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use virtual_trading::VirtualTradingSystem;
use heartbeat::Heartbeat;
use strategy_core::node::context_trait::NodeMetaDataExt;



pub type PositionNodeMetadata = NodeMetadata<
    PositionNodeStateMachine,
    BacktestNodeEvent,
    BacktestNodeCommand,
    BacktestStrategyCommand
>;


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