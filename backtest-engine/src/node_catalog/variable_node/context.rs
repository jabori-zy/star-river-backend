mod benchmark;
mod config_filter;
mod custom_variable_handler;
mod event_handler;
mod node_handles;
mod sys_variable_handler;
mod variable_handler;

use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use star_river_core::custom_type::NodeId;
use strategy_core::{
    node::{context_trait::NodeMetaDataExt, metadata::NodeMetadata},
    variable::custom_variable::VariableValue,
};
use tokio::sync::{Mutex, RwLock};

use super::{state_machine::VariableNodeStateMachine, variable_node_type::VariableNodeBacktestConfig};
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::VariableNodeError, node_event::BacktestNodeEvent},
    strategy::strategy_command::BacktestStrategyCommand,
    virtual_trading_system::BacktestVts,
};

pub type VariableNodeMetadata = NodeMetadata<VariableNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct VariableNodeContext {
    metadata: VariableNodeMetadata,
    node_config: VariableNodeBacktestConfig,
    virtual_trading_system: Arc<Mutex<BacktestVts>>,
    variable_cache_value: Arc<RwLock<HashMap<(NodeId, i32, String), VariableValue>>>,
}

impl VariableNodeContext {
    pub fn new(
        metadata: VariableNodeMetadata,
        node_config: VariableNodeBacktestConfig,
        virtual_trading_system: Arc<Mutex<BacktestVts>>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            virtual_trading_system,
            variable_cache_value: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl NodeMetaDataExt for VariableNodeContext {
    type StateMachine = VariableNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;
    type Error = VariableNodeError;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

impl VariableNodeContext {
    pub async fn update_variable_cache_value(
        &mut self,
        node_id: NodeId,
        config_id: i32,
        variable_name: String,
        variable_value: VariableValue,
    ) {
        let mut variable_cache_value_guard = self.variable_cache_value.write().await;
        variable_cache_value_guard.insert((node_id, config_id, variable_name), variable_value);
    }

    pub async fn get_variable_cache_value(&mut self, node_id: NodeId, config_id: i32, variable_name: String) -> Option<VariableValue> {
        let variable_cache_value_guard = self.variable_cache_value.read().await;
        variable_cache_value_guard.get(&(node_id, config_id, variable_name)).cloned()
    }
}
