mod benchmark;
mod node_handles;
mod event_handler;
mod config_filter;
mod custom_variable_handler;
mod sys_variable_handler;
mod variable_handler;

use star_river_core::custom_type::NodeId;
use strategy_core::node::metadata::NodeMetadata;
use strategy_core::variable::custom_variable::VariableValue;
use crate::node::node_event::BacktestNodeEvent;
use crate::node::node_command::BacktestNodeCommand;
use crate::strategy::strategy_command::BacktestStrategyCommand;
use super::state_machine::VariableNodeStateMachine;
use super::variable_node_type::VariableNodeBacktestConfig;
use crate::strategy::PlayIndex;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;
use virtual_trading::VirtualTradingSystem;
use heartbeat::Heartbeat;
use strategy_core::node::context_trait::NodeMetaDataExt;
use std::collections::HashMap;
use tokio::sync::RwLock;



pub type VariableNodeMetadata = NodeMetadata<
    VariableNodeStateMachine,
    BacktestNodeEvent,
    BacktestNodeCommand,
    BacktestStrategyCommand
>;


#[derive(Debug)]
pub struct VariableNodeContext {
    metadata: VariableNodeMetadata,
    node_config: VariableNodeBacktestConfig,
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    variable_cache_value: Arc<RwLock<HashMap<(NodeId, i32, String), VariableValue>>>,
}


impl VariableNodeContext {
    pub fn new(
        metadata: VariableNodeMetadata,
        node_config: VariableNodeBacktestConfig,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            play_index_watch_rx,
            virtual_trading_system,
            variable_cache_value: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl VariableNodeContext {
    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }
}


impl NodeMetaDataExt for VariableNodeContext {
    type StateMachine = VariableNodeStateMachine;
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