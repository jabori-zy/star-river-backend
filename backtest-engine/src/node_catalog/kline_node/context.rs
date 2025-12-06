mod event_handler;
mod node_handles;

mod binance_data_handler;
// mod command_handler;
mod data_handler;
mod mt5_data_handler;
mod status_handler;
mod utils;
// mod node_handles;

use std::{collections::HashMap, fmt::Debug};

use async_trait::async_trait;
use key::KlineKey;
use star_river_core::{
    custom_type::{NodeId, NodeName},
    kline::KlineInterval,
};
use strategy_core::{
    benchmark::node_benchmark::CompletedCycle,
    node::{
        context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeMetaDataExt},
        metadata::NodeMetadata,
    },
};

use super::{kline_node_type::KlineNodeBacktestConfig, state_machine::KlineNodeStateMachine};
use crate::{
    node::{node_command::BacktestNodeCommand, node_error::kline_node_error::*, node_event::BacktestNodeEvent},
    strategy::strategy_command::BacktestStrategyCommand,
};

pub type KlineNodeMetadata = NodeMetadata<KlineNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct KlineNodeContext {
    pub metadata: KlineNodeMetadata,
    pub node_config: KlineNodeBacktestConfig,
    min_interval: KlineInterval,
    selected_symbol_keys: HashMap<KlineKey, (i32, String)>, // Configured symbol keys -> (config_id, output_handle_id)
    correct_index: u64,                                     // Correct index (needs correction if index is incorrect)
}

impl KlineNodeContext {
    pub fn new(metadata: KlineNodeMetadata, node_config: KlineNodeBacktestConfig) -> Result<Self, KlineNodeError> {
        let exchange = node_config.exchange_mode()?.selected_account.exchange.clone();
        let time_range = node_config.exchange_mode()?.time_range.clone();

        let selected_symbol_keys = node_config
            .exchange_mode()?
            .selected_symbols
            .iter()
            .map(|symbol| {
                let kline_key = KlineKey::new(
                    exchange.clone(),
                    symbol.symbol.clone(),
                    symbol.interval.clone(),
                    Some(time_range.start_date.to_string()),
                    Some(time_range.end_date.to_string()),
                );
                (kline_key, (symbol.config_id, symbol.output_handle_id.clone()))
            })
            .collect();

        Ok(Self {
            metadata,
            node_config,
            min_interval: KlineInterval::Minutes1,
            selected_symbol_keys,
            correct_index: 0,
        })
    }

    pub fn min_interval(&self) -> &KlineInterval {
        &self.min_interval
    }

    pub fn set_min_interval(&mut self, interval: KlineInterval) {
        self.min_interval = interval;
    }

    pub fn selected_symbol_keys(&self) -> &HashMap<KlineKey, (i32, String)> {
        &self.selected_symbol_keys
    }
}

impl NodeMetaDataExt for KlineNodeContext {
    type StateMachine = KlineNodeStateMachine;
    type NodeEvent = BacktestNodeEvent;
    type NodeCommand = BacktestNodeCommand;
    type StrategyCommand = BacktestStrategyCommand;
    type Error = KlineNodeError;

    fn metadata(&self) -> &NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut NodeMetadata<Self::StateMachine, Self::NodeEvent, Self::NodeCommand, Self::StrategyCommand> {
        &mut self.metadata
    }
}

#[async_trait]
impl NodeBenchmarkExt for KlineNodeContext {
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
