// std
use std::collections::HashMap;

// third-party
use chrono::Utc;
// workspace crate
use key::{Key, KeyTrait, KlineKey};
use star_river_core::kline::Kline;
use strategy_core::strategy::context_trait::StrategyIdentityExt;
use virtual_trading::vts_trait::VtsCtxAccessor;

// current crate
use super::BacktestStrategyContext;
use crate::strategy::strategy_error::{BacktestStrategyError, IntervalNotSameSnafu};

// 检查策略的symbol配置
impl BacktestStrategyContext {
    pub async fn check_symbol_config(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.strategy_name();
        tracing::info!("[{}] start check symbol config", strategy_name);

        let keys = self.keys().await;

        let mut min_symbol_map: HashMap<String, KlineKey> = HashMap::new();
        for (key, _) in keys {
            if let Key::Kline(kline_key) = key {
                let symbol = kline_key.get_symbol();
                let interval = kline_key.get_interval();
                // 是否需要替换（如果存在，且interval小于已存在的，则替换）
                let should_replace = min_symbol_map
                    .get(&symbol)
                    .map(|existing| interval < existing.get_interval())
                    .unwrap_or(true);
                if should_replace {
                    min_symbol_map.insert(symbol, kline_key);
                }
            }
        }

        if min_symbol_map.is_empty() {
            tracing::warn!("[{}] no kline symbols configured", strategy_name);
            self.set_min_interval_symbols(Vec::new());
            self.virtual_trading_system
                .lock()
                .await
                .with_ctx_write(|ctx| ctx.set_kline_price(HashMap::<KlineKey, Kline>::new()))
                .await;
            return Ok(());
        }

        let min_interval_symbols: Vec<KlineKey> = min_symbol_map.into_values().collect();
        let reference_interval = min_interval_symbols[0].get_interval();
        if min_interval_symbols.iter().any(|key| key.get_interval() != reference_interval) {
            return Err(IntervalNotSameSnafu {
                symbols: min_interval_symbols
                    .iter()
                    .map(|key| (key.get_symbol(), key.get_interval().to_string()))
                    .collect::<Vec<(String, String)>>(),
            }
            .build());
        }
        tracing::debug!("set min interval symbols: {:?}", min_interval_symbols);
        self.set_min_interval_symbols(min_interval_symbols.clone());

        let mut kline_price = HashMap::with_capacity(min_interval_symbols.len());
        for kline_key in min_interval_symbols {
            kline_price.insert(
                kline_key,
                Kline {
                    datetime: Utc::now(),
                    open: 0.0,
                    high: 0.0,
                    low: 0.0,
                    close: 0.0,
                    volume: 0.0,
                },
            );
        }
        self.virtual_trading_system()
            .lock()
            .await
            .with_ctx_write(|ctx| ctx.set_kline_price(kline_price))
            .await;
        Ok(())
    }
}
