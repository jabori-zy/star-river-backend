mod binance_data_handler;
mod command_handler;
mod data_handler;
mod event_handler;
mod mt5_data_handler;
mod status_handler;
mod utils;
mod node_handles;

use super::kline_node_type::KlineNodeBacktestConfig;
use event_center::EventCenterSingleton;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::*;
use event_center::communication::engine::EngineResponse;
use event_center::communication::engine::exchange_engine::{
    ExchangeEngineCommand, RegisterExchangeCmdPayload, RegisterExchangeCommand, RegisterExchangeRespPayload,
};
use event_center::communication::engine::market_engine::{GetKlineHistoryCmdPayload, GetKlineHistoryCommand, MarketEngineCommand};
use event_center::event::node_event::backtest_node_event::kline_node_event::{KlineNodeEvent, KlineUpdateEvent, KlineUpdatePayload};
use star_river_core::custom_type::AccountId;
use star_river_core::custom_type::PlayIndex;
use crate::error::node_error::kline_node_error::*;
use star_river_core::key::KeyTrait;
use star_river_core::key::key::KlineKey;
use star_river_core::market::KlineInterval;
use star_river_core::market::{Exchange, Kline};
use star_river_core::strategy::TimeRange;
use star_river_core::strategy::node_benchmark::CycleTracker;
use std::collections::HashMap;
use std::fmt::Debug;

use crate::node::base_context::NodeBaseContext;
use crate::node_list::kline_node::state_machine::KlineNodeAction;
use crate::node::node_context_trait::NodeBaseContextTrait;

#[derive(Debug, Clone)]
pub struct KlineNodeContext {
    pub base_context: NodeBaseContext<KlineNodeAction>,
    pub node_config: KlineNodeBacktestConfig,
    min_interval_symbols: Vec<KlineKey>,
    selected_symbol_keys: HashMap<KlineKey, (i32, String)>, // 已配置的symbol键 -> (配置id, 输出句柄id)
}

impl KlineNodeContext {
    pub fn new(base_context: NodeBaseContext<KlineNodeAction>, node_config: KlineNodeBacktestConfig) -> Self {
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
            base_context,
            node_config,
            min_interval_symbols: vec![],
            selected_symbol_keys,
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



impl NodeBaseContextTrait<KlineNodeAction> for KlineNodeContext {
    fn base_context(&self) -> &NodeBaseContext<KlineNodeAction> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut NodeBaseContext<KlineNodeAction> {
        &mut self.base_context
    }
}
