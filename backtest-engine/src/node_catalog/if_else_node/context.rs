mod condition_op;
mod event_handler;
mod node_handles;

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use star_river_core::custom_type::{NodeId, NodeName};
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    node::{
        context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeMetaDataExt},
        metadata::NodeMetadata,
    },
};

use super::{if_else_node_type::IfElseNodeBacktestConfig, state_machine::IfElseNodeStateMachine};
use crate::{
    node::{node_command::BacktestNodeCommand, node_event::BacktestNodeEvent},
    strategy::strategy_command::BacktestStrategyCommand,
};

pub type ConfigId = i32;

pub type IfElseNodeMetadata = NodeMetadata<IfElseNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct IfElseNodeContext {
    metadata: IfElseNodeMetadata,
    node_config: IfElseNodeBacktestConfig,
    received_flag: HashMap<(NodeId, ConfigId), bool>, // 用于记录每个variable的数据是否接收
    received_message: HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>, // 用于记录每个variable的数据(node_id + variable_id)为key
    is_nested: bool,
    superior_case_is_true: bool,
}

impl IfElseNodeContext {
    pub fn new(metadata: IfElseNodeMetadata, node_config: IfElseNodeBacktestConfig, is_nested: bool) -> Self {
        Self {
            metadata,
            node_config,
            received_flag: HashMap::new(),
            received_message: HashMap::new(),
            is_nested,
            superior_case_is_true: false,
        }
    }
}

impl IfElseNodeContext {
    pub fn is_nested(&self) -> bool {
        self.is_nested
    }

    pub fn superior_case_is_true(&self) -> bool {
        self.superior_case_is_true
    }

    pub fn set_superior_case_is_true(&mut self, superior_case_is_true: bool) {
        self.superior_case_is_true = superior_case_is_true;
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

#[async_trait]
impl NodeBenchmarkExt for IfElseNodeContext {
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
