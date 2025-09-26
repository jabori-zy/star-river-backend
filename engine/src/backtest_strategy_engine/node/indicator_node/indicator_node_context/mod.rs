mod context_impl;
mod data_handler;
mod event_handler;
mod status_handler;

use super::indicator_node_type::IndicatorNodeBacktestConfig;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use event_center::EventCenterSingleton;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::*;
use event_center::communication::engine::indicator_engine::{
    CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorCommand, GetIndicatorLookbackCmdPayload, GetIndicatorLookbackCommand,
    IndicatorEngineCommand,
};
use star_river_core::indicator::Indicator;
use star_river_core::key::KeyTrait;
use star_river_core::key::key::{IndicatorKey, KlineKey};
use star_river_core::market::Kline;
use star_river_core::market::QuantData;
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct IndicatorNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub backtest_config: IndicatorNodeBacktestConfig,
    selected_kline_key: KlineKey,                         // 回测K线缓存键
    indicator_keys: HashMap<IndicatorKey, (i32, String)>, // 指标缓存键 -> (配置id, 输出句柄id)
    kline_value: HashMap<IndicatorKey, Vec<Kline>>,       // 指标缓存键 -> 指标值
    indicator_lookback: HashMap<IndicatorKey, usize>,     // 指标缓存键 -> lookback
    min_interval_symbols: Vec<KlineKey>,
}

impl IndicatorNodeContext {
    pub fn new(
        base_context: BacktestBaseNodeContext,
        backtest_config: IndicatorNodeBacktestConfig,
        selected_kline_key: KlineKey,
        indicator_keys: HashMap<IndicatorKey, (i32, String)>,
    ) -> Self {
        Self {
            base_context,
            backtest_config,
            selected_kline_key,
            indicator_keys,
            kline_value: HashMap::new(),
            indicator_lookback: HashMap::new(),
            min_interval_symbols: vec![],
        }
    }

    pub fn set_min_interval_symbols(&mut self, min_interval_symbols: Vec<KlineKey>) {
        self.min_interval_symbols = min_interval_symbols;
    }

    pub fn get_min_interval_symbols_ref(&self) -> &Vec<KlineKey> {
        &self.min_interval_symbols
    }

    pub fn get_indicator_keys_ref(&self) -> &HashMap<IndicatorKey, (i32, String)> {
        &self.indicator_keys
    }
}
