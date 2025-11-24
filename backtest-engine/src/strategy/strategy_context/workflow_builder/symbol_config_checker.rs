// std
use std::collections::HashMap;

// third-party
use chrono::Utc;
// workspace crate
use key::{Key, KeyTrait, KlineKey};
use star_river_core::kline::{Kline, KlineInterval};
use strategy_core::strategy::context_trait::StrategyIdentityExt;
use virtual_trading::vts_trait::VtsCtxAccessor;

// current crate
use super::BacktestStrategyContext;
use crate::strategy::strategy_error::{BacktestStrategyError, IntervalNotSameSnafu, NoSymbolConfiguredSnafu};

// 检查策略的symbol配置
impl BacktestStrategyContext {
    pub async fn check_symbol_config(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.strategy_name();
        tracing::info!("[{}] start check symbol config", strategy_name);

        let keys = self.keys().await;

        let mut min_symbol_map: HashMap<String, KlineKey> = HashMap::new();

        for key in keys.keys() {
            if let Key::Kline(kline_key) = key {
                let symbol = kline_key.symbol();
                let interval = kline_key.interval();
                // 是否需要替换（如果存在，且interval小于已存在的，则替换）
                let should_replace = min_symbol_map
                    .get(&symbol)
                    .map(|existing| interval < existing.interval())
                    .unwrap_or(true);
                if should_replace {
                    min_symbol_map.insert(symbol, kline_key.clone());
                }
            }
        }

        let min_interval_symbols: Vec<KlineKey> = min_symbol_map.into_values().collect();
        if let Some(reference_symbol) = min_interval_symbols.first() {
            let reference_interval = reference_symbol.interval();
            if min_interval_symbols.iter().any(|key| key.interval() != reference_interval) {
                return Err(IntervalNotSameSnafu {
                    symbols: min_interval_symbols
                        .iter()
                        .map(|key| (key.symbol(), key.interval().to_string()))
                        .collect::<Vec<(String, String)>>(),
                }
                .build());
            }
            self.set_min_interval(reference_interval);
        } else {
            return Err(NoSymbolConfiguredSnafu {
                strategy_name: strategy_name.to_string(),
            }
            .build());
        }

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
