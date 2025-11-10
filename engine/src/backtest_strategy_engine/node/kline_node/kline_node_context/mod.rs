mod command_handler;
mod context_impl;
mod data_handler;
mod event_handler;
mod status_handler;
mod utils;
mod mt5_data_handler;
mod binance_data_handler;

use super::kline_node_type::KlineNodeBacktestConfig;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use event_center::EventCenterSingleton;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::*;
use event_center::communication::engine::EngineResponse;
use event_center::communication::engine::exchange_engine::{
    ExchangeEngineCommand, RegisterExchangeCmdPayload, RegisterExchangeCommand, RegisterExchangeRespPayload,
};
use event_center::communication::engine::market_engine::{GetKlineHistoryCmdPayload, GetKlineHistoryCommand, MarketEngineCommand};
use event_center::event::node_event::backtest_node_event::kline_node_event::{KlineNodeEvent, KlineUpdateEvent, KlineUpdatePayload};
use star_river_core::custom_type::PlayIndex;
use star_river_core::error::engine_error::node_error::backtest_strategy_node_error::kline_node_error::*;
use star_river_core::key::KeyTrait;
use star_river_core::key::key::KlineKey;
use star_river_core::market::{Kline, Exchange};
use star_river_core::system::TimeRange;
use std::collections::HashMap;
use std::fmt::Debug;
use star_river_core::custom_type::AccountId;
use star_river_core::market::KlineInterval;
use star_river_core::strategy::node_benchmark::CycleTracker;

#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub node_config: KlineNodeBacktestConfig,
    min_interval_symbols: Vec<KlineKey>,
    selected_symbol_keys: HashMap<KlineKey, (i32, String)>, // 已配置的symbol键 -> (配置id, 输出句柄id)
}

impl KlineNodeContext {
    pub fn new(
        base_context: BacktestBaseNodeContext, 
        node_config: KlineNodeBacktestConfig) -> Self {
        let exchange = node_config
            .exchange_mode_config
            .as_ref()
            .unwrap()
            .selected_account
            .exchange
            .clone();
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
            base_context,
            node_config,
            min_interval_symbols: vec![],
            selected_symbol_keys,
        }
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn get_selected_symbol_keys_ref(&self) -> &HashMap<KlineKey, (i32, String)> {
        &self.selected_symbol_keys
    }
}
