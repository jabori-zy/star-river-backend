mod benchmark;
mod condition_op;
mod event_handler;
mod node_handles;

use std::collections::HashMap;

use star_river_core::custom_type::NodeId;
use strategy_core::node::{context_trait::NodeMetaDataExt, metadata::NodeMetadata};

use super::{if_else_node_type::IfElseNodeBacktestConfig, state_machine::IfElseNodeStateMachine};
use crate::{
    node::{node_command::BacktestNodeCommand, node_event::BacktestNodeEvent},
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
};

pub type ConfigId = i32;

pub type IfElseNodeMetadata = NodeMetadata<IfElseNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct IfElseNodeContext {
    metadata: IfElseNodeMetadata,
    node_config: IfElseNodeBacktestConfig,
    received_flag: HashMap<(NodeId, ConfigId), bool>, // 用于记录每个variable的数据是否接收
    received_message: HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>, // 用于记录每个variable的数据(node_id + variable_id)为key
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
}

impl IfElseNodeContext {
    pub fn new(
        metadata: IfElseNodeMetadata,
        node_config: IfElseNodeBacktestConfig,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            received_flag: HashMap::new(),
            received_message: HashMap::new(),
            play_index_watch_rx,
        }
    }
}

impl IfElseNodeContext {
    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }
}

impl NodeMetaDataExt for IfElseNodeContext {
    type StateMachine = IfElseNodeStateMachine;
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
