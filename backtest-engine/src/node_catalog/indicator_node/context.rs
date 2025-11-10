mod benchmark;
mod data_handler;
mod event_handler;
mod node_handles;
mod status_handler;

// Standard library imports
use std::{collections::HashMap, fmt::Debug};

// External project crates
use key::{IndicatorKey, KlineKey};
use star_river_core::kline::Kline;
use strategy_core::node::{context_trait::NodeMetaDataExt, metadata::NodeMetadata};

// Local module imports
use super::{indicator_node_type::IndicatorNodeBacktestConfig, state_machine::IndicatorNodeStateMachine};
// Crate imports
use crate::{
    node::{node_command::BacktestNodeCommand, node_event::BacktestNodeEvent},
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
};

pub type IndicatorNodeMetadata = NodeMetadata<IndicatorNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct IndicatorNodeContext {
    metadata: IndicatorNodeMetadata,
    node_config: IndicatorNodeBacktestConfig,
    selected_kline_key: KlineKey,                         // Selected kline cache key
    indicator_keys: HashMap<IndicatorKey, (i32, String)>, // Indicator key -> (config_id, output_handle_id)
    kline_value: HashMap<IndicatorKey, Vec<Kline>>,       // Indicator key -> kline values
    indicator_lookback: HashMap<IndicatorKey, usize>,     // Indicator key -> lookback
    min_interval_symbols: Vec<KlineKey>,
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
}

impl IndicatorNodeContext {
    pub fn new(
        metadata: IndicatorNodeMetadata,
        node_config: IndicatorNodeBacktestConfig,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        selected_kline_key: KlineKey,
        indicator_keys: HashMap<IndicatorKey, (i32, String)>,
    ) -> Self {
        Self {
            metadata,
            node_config,
            selected_kline_key,
            indicator_keys,
            kline_value: HashMap::new(),
            indicator_lookback: HashMap::new(),
            min_interval_symbols: vec![],
            play_index_watch_rx,
        }
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn min_interval_symbols(&self) -> &Vec<KlineKey> {
        &self.min_interval_symbols
    }

    pub fn indicator_keys(&self) -> &HashMap<IndicatorKey, (i32, String)> {
        &self.indicator_keys
    }
}

impl IndicatorNodeContext {
    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }
}

impl NodeMetaDataExt for IndicatorNodeContext {
    type StateMachine = IndicatorNodeStateMachine;
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
