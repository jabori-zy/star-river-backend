mod data_handler;
mod event_handler;
mod node_handles;
mod status_handler;

// Standard library imports
use std::{collections::HashMap, fmt::Debug};

// External project crates
use async_trait::async_trait;
use key::{IndicatorKey, KlineKey};
use star_river_core::{
    custom_type::{NodeId, NodeName},
    kline::{Kline, KlineInterval},
};
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    node::{
        context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeMetaDataExt},
        metadata::NodeMetadata,
    },
};

// Local module imports
use super::{indicator_node_type::IndicatorNodeBacktestConfig, state_machine::IndicatorNodeStateMachine};
// Crate imports
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::IndicatorNodeError, node_event::BacktestNodeEvent, node_utils::NodeUtils},
    strategy::strategy_command::BacktestStrategyCommand,
};

pub type IndicatorNodeMetadata = NodeMetadata<IndicatorNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct IndicatorNodeContext {
    metadata: IndicatorNodeMetadata,
    node_config: IndicatorNodeBacktestConfig,
    selected_kline_key: KlineKey,                         // Selected kline cache key
    indicator_keys: HashMap<IndicatorKey, (i32, String)>, // Indicator key -> (config_id, output_handle_id)
    cache_kline_slice: HashMap<IndicatorKey, Vec<Kline>>, // Indicator key -> kline values
    indicator_lookback: HashMap<IndicatorKey, usize>,     // Indicator key -> lookback
    min_interval: KlineInterval,
    correct_index: u64, // Correct index
}

impl IndicatorNodeContext {
    pub fn new(
        metadata: IndicatorNodeMetadata,
        node_config: IndicatorNodeBacktestConfig,
        selected_kline_key: KlineKey,
        indicator_keys: HashMap<IndicatorKey, (i32, String)>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            selected_kline_key,
            indicator_keys,
            cache_kline_slice: HashMap::new(),
            indicator_lookback: HashMap::new(),
            min_interval: KlineInterval::Months1,
            correct_index: 0,
        }
    }

    pub fn min_interval(&self) -> &KlineInterval {
        &self.min_interval
    }

    pub fn set_min_interval(&mut self, interval: KlineInterval) {
        self.min_interval = interval;
    }

    pub fn indicator_keys(&self) -> &HashMap<IndicatorKey, (i32, String)> {
        &self.indicator_keys
    }
}

impl NodeMetaDataExt for IndicatorNodeContext {
    type StateMachine = IndicatorNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;
    type Error = IndicatorNodeError;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

#[async_trait]
impl NodeBenchmarkExt for IndicatorNodeContext {
    async fn mount_node_cycle_tracker(
        &self,
        node_id: NodeId,
        node_name: NodeName,
        cycle_tracker: CompletedCycle,
    ) -> Result<(), Self::Error> {
        NodeUtils::mount_node_cycle_tracker(node_id, node_name, cycle_tracker, self.strategy_command_sender()).await?;
        Ok(())
    }
}
