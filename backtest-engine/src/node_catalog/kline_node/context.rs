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
use star_river_core::custom_type::NodeId;
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
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
};

pub type KlineNodeMetadata = NodeMetadata<KlineNodeStateMachine, BacktestNodeEvent, BacktestNodeCommand, BacktestStrategyCommand>;

#[derive(Debug)]
pub struct KlineNodeContext {
    pub metadata: KlineNodeMetadata,
    pub node_config: KlineNodeBacktestConfig,
    min_interval_symbols: Vec<KlineKey>,
    selected_symbol_keys: HashMap<KlineKey, (i32, String)>, // 已配置的symbol键 -> (配置id, 输出句柄id)
    play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
}

impl KlineNodeContext {
    pub fn new(
        metadata: KlineNodeMetadata,
        node_config: KlineNodeBacktestConfig,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Self {
        let exchange = node_config.exchange_mode_config.as_ref().unwrap().selected_account.exchange.clone();
        let time_range = node_config.exchange_mode_config.as_ref().unwrap().time_range.clone();

        let selected_symbol_keys = node_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
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

        Self {
            metadata,
            node_config,
            min_interval_symbols: vec![],
            selected_symbol_keys,
            play_index_watch_rx,
        }
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        // tracing::debug!("set min interval symbols: {:?}", min_interval_symbols);
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn selected_symbol_keys(&self) -> &HashMap<KlineKey, (i32, String)> {
        &self.selected_symbol_keys
    }
}

impl KlineNodeContext {
    pub fn play_index(&self) -> PlayIndex {
        *self.play_index_watch_rx.borrow()
    }

    pub fn play_index_watch_rx(&self) -> &tokio::sync::watch::Receiver<PlayIndex> {
        &self.play_index_watch_rx
    }
}

impl NodeMetaDataExt for KlineNodeContext {
    type StateMachine = KlineNodeStateMachine;
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
impl NodeBenchmarkExt for KlineNodeContext {
    type Error = crate::node::node_error::BacktestNodeError;

    async fn mount_node_cycle_tracker(&self, node_id: NodeId, cycle_tracker: CompletedCycle) -> Result<(), Self::Error> {
        crate::node::node_utils::NodeUtils::mount_node_cycle_tracker(node_id, cycle_tracker, self.strategy_command_sender()).await?;
        Ok(())
    }
}
