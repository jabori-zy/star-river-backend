mod benchmark;
mod event_handler;
mod node_handles;

mod binance_data_handler;
// mod command_handler;
mod data_handler;
mod mt5_data_handler;
mod status_handler;
mod utils;
// mod node_handles;

use super::config::KlineNodeBacktestConfig;
use star_river_core::custom_type::AccountId;
use star_river_core::custom_type::PlayIndex;
use crate::error::node_error::kline_node_error::*;
use key::{KeyTrait, KlineKey};
use star_river_core::kline::{KlineInterval, Kline};
use star_river_core::exchange::{Exchange};
use strategy_core::strategy::TimeRange;
use strategy_core::benchmark::node_benchmark::CycleTracker;
use std::collections::HashMap;
use std::fmt::Debug;

use strategy_core::node::metadata::NodeMetadata;
use super::state_machine::KlineNodeAction;
use super::state_machine::KlineNodeStateMachine;
use crate::node_event::BacktestNodeEvent;
use crate::strategy_command::BacktestStrategyCommand;
use crate::node_command::BacktestNodeCommand;
use strategy_core::node::context_trait::NodeMetaDataExt;

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
        tracing::debug!("set min interval symbols: {:?}", min_interval_symbols);
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
