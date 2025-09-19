use super::{
    BacktestStrategy, BacktestStrategyError,
    BacktestStrategyFunction,
    NodeCommand,
    StrategyInnerEvent,
    Key,
    KlineInterval,
};
use tokio::sync::{mpsc, broadcast};
use star_river_core::error::engine_error::strategy_error::backtest_strategy_error::*;
use snafu::IntoError;
use std::collections::HashMap;


impl BacktestStrategy {
    pub async fn add_node(&mut self) -> Result<(), BacktestStrategyError> {
        let (node_command_tx, node_command_rx) = mpsc::channel::<NodeCommand>(100);
        let (strategy_inner_event_tx, strategy_inner_event_rx) =
            broadcast::channel::<StrategyInnerEvent>(100);

        // setting strategy context properties
        {
            let mut context_guard = self.context.write().await;
            context_guard.set_node_command_receiver(node_command_rx);
            context_guard.set_strategy_inner_event_publisher(strategy_inner_event_tx);
        } // context lock end

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

        let context = self.get_context();
        for node_config in node_config_list {
            let result = BacktestStrategyFunction::add_node(
                context.clone(),
                node_config,
                node_command_tx.clone(),
                strategy_inner_event_rx.resubscribe(),
            )
            .await;
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
            BacktestStrategyFunction::add_edge(context.clone(), edge_config)
                .await
                .unwrap();
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
        let context = self.get_context();

        // 先获取keys，然后立即释放读锁
        let keys = {
            let context_guard = context.read().await;
            context_guard.get_keys().await
        }; // 读锁在这里被释放

        // 检查不同symbol的最小interval是否相同

        // 1. 按symbol分组
        let mut symbol_groups = HashMap::new();
        for key in keys.iter() {
            if matches!(key, Key::Kline(_)) {
                let symbol = key.get_symbol();
                symbol_groups.entry(symbol).or_insert_with(Vec::new).push(key);
            }
        }
        // tracing::debug!("symbol_groups: {:#?}", symbol_groups);

        // 2. 检查key(symbol) 的个数
        // 如果只有一个symbol，则直接返回
        if symbol_groups.keys().len() == 1 {
            // 如果只有一个symbol，则找到最小的interval的key
            if let Some(keys) = symbol_groups.values().next() {
                if let Some(&min_interval_symbol) = keys.iter().min_by_key(|key| key.get_interval()) {
                    let mut context_guard = context.write().await;
                    context_guard.set_min_interval_symbols(vec![min_interval_symbol.clone()]);
                    return Ok(());
                }
            }
            // 如果无法找到有效的symbol或key，返回错误
            return Err(IntervalNotSameSnafu {
                symbols: vec![],
            }.build());
        }

        // 3. 获取每个symbol组内interval最小的key
        let min_interval_symbols: Vec<&Key> = symbol_groups
            .values()
            .filter_map(|keys| {keys.iter().min_by_key(|key| key.get_interval()).copied()
            })
            .collect();
        // tracing::debug!("min_interval_symbols: {:#?}", min_interval_symbols);

        // 4. 检查列表内的所有interval是否相同.(去重之后是否等于1)
        let min_interval_intervals = min_interval_symbols.iter().map(|key| key.get_interval()).collect::<Vec<KlineInterval>>();
        let min_interval_intervals_unique = if min_interval_intervals.is_empty() {
            0
        } else {
            let first_interval = &min_interval_intervals[0];
            if min_interval_intervals.iter().all(|interval| interval == first_interval) {
                1  // 所有元素都相同
            } else {
                2  // 存在不同元素（大于1即可）
            }
        };

        // tracing::debug!("min_interval_intervals: {:#?}", min_interval_intervals);
        // tracing::debug!("unique interval count: {}", min_interval_intervals_unique);

        // 如果去重后的interval数量大于1，说明不同symbol的最小interval不相同
        if min_interval_intervals_unique > 1 {
            return Err(IntervalNotSameSnafu {
                symbols: min_interval_symbols.iter().map(|key| (key.get_symbol(), key.get_interval().to_string())).collect::<Vec<(String, String)>>(),
            }.build());
        }

        // 最后获取写锁并设置结果
        let mut context_guard = context.write().await;
        context_guard.set_min_interval_symbols(min_interval_symbols.into_iter().cloned().collect());
        Ok(())
    }
}