use super::{BacktestStrategy, BacktestStrategyError, BacktestStrategyFunction, Key, KeyTrait, KlineInterval, KlineKey};
use chrono::Utc;
use snafu::IntoError;
use star_river_core::{error::engine_error::strategy_error::backtest_strategy_error::*, market::Kline};
use std::collections::HashMap;
use tokio::sync::{broadcast, mpsc};

impl BacktestStrategy {
    pub async fn add_node(&mut self) -> Result<(), BacktestStrategyError> {
        // get strategy config
        let node_config_list = {
            let context_guard = self.context.read().await;
            let node_config_list = context_guard
                .strategy_config
                .nodes
                .as_ref()
                .and_then(|node| node.as_array())
                .ok_or_else(|| {
                    NodeConfigNullSnafu {
                        strategy_id: context_guard.strategy_id,
                        strategy_name: context_guard.strategy_name.clone(),
                    }
                    .build()
                })?;
            node_config_list.clone()
        };

        let strategy_command_tx = {
            let context = self.get_context();
            let context_guard = context.read().await;
            context_guard.strategy_command_sender.clone()
        };
        let context = self.get_context();
        for node_config in node_config_list {
            let result = BacktestStrategyFunction::add_node(context.clone(), node_config, strategy_command_tx.clone()).await;
            if let Err(e) = result {
                let error = NodeCheckSnafu {}.into_error(e);
                return Err(error);
            }
        }

        Ok(())
    }

    pub async fn add_edge(&mut self) -> Result<(), BacktestStrategyError> {
        let context = self.get_context();

        let edge_config_list = {
            let context_guard = context.read().await;
            context_guard
                .strategy_config
                .edges
                .as_ref()
                .and_then(|edge| edge.as_array())
                .ok_or_else(|| {
                    EdgeConfigNullSnafu {
                        strategy_id: context_guard.strategy_id,
                        strategy_name: context_guard.strategy_name.clone(),
                    }
                    .build()
                })?
                .clone()
        };

        for edge_config in edge_config_list {
            BacktestStrategyFunction::add_edge(context.clone(), edge_config).await.unwrap();
        }
        Ok(())
    }

    pub async fn set_leaf_nodes(&mut self) -> Result<(), BacktestStrategyError> {
        let context = self.get_context();
        BacktestStrategyFunction::set_leaf_nodes(context).await;
        Ok(())
    }

    pub async fn set_strategy_output_handles(&mut self) -> Result<(), BacktestStrategyError> {
        let context = self.get_context();
        BacktestStrategyFunction::add_strategy_output_handle(context).await;
        Ok(())
    }

    // 检查策略的symbol配置
    pub async fn check_symbol_config(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        tracing::info!("[{}] start check symbol config", strategy_name);
        let context = self.get_context();

        let keys = {
            let context_guard = context.read().await;
            context_guard.get_keys().await
        };

        let mut min_symbol_map: HashMap<String, KlineKey> = HashMap::new();
        for (key, _) in keys {
            if let Key::Kline(kline_key) = key {
                let symbol = kline_key.get_symbol();
                let interval = kline_key.get_interval();
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
            let mut context_guard = context.write().await;
            context_guard.set_min_interval_symbols(Vec::new());
            context_guard
                .virtual_trading_system
                .lock()
                .await
                .set_kline_price(HashMap::<KlineKey, Kline>::new());
            return Ok(());
        }

        if min_symbol_map.len() == 1 {
            tracing::info!("[{}] have only one symbol", strategy_name);
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

        let mut context_guard = context.write().await;
        context_guard.set_min_interval_symbols(min_interval_symbols.clone());

        let mut virtual_trading_system_guard = context_guard.virtual_trading_system.lock().await;
        let now = Utc::now();
        let mut kline_price = HashMap::with_capacity(min_interval_symbols.len());
        for kline_key in min_interval_symbols {
            kline_price.insert(
                kline_key,
                Kline {
                    datetime: now,
                    open: 0.0,
                    high: 0.0,
                    low: 0.0,
                    close: 0.0,
                    volume: 0.0,
                },
            );
        }
        virtual_trading_system_guard.set_kline_price(kline_price);
        Ok(())
    }
}
