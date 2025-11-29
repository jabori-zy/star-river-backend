use chrono::{DateTime, Utc};
// workspace crate
use key::{Key, KeyTrait};
use strategy_core::strategy::context_trait::StrategyIdentityExt;
use strategy_stats::StatsSnapshot;
use virtual_trading::{
    types::{VirtualOrder, VirtualPosition, VirtualTransaction},
    vts_trait::VtsCtxAccessor,
};

// current crate
use super::BacktestStrategyContext;
use crate::strategy::strategy_error::{BacktestStrategyError, GetDataFailedSnafu};

impl BacktestStrategyContext {
    // 获取所有的virtual order
    pub async fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let virtual_orders = virtual_trading_system
            .with_ctx_read(|ctx| {
                ctx.unfilled_orders()
                    .clone()
                    .into_iter()
                    .chain(ctx.history_orders().clone())
                    .collect()
            })
            .await;
        virtual_orders
    }

    pub async fn get_current_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let current_positions = virtual_trading_system
            .with_ctx_read(|ctx| ctx.get_current_positions().clone())
            .await;
        current_positions
    }

    pub async fn get_history_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let history_positions = virtual_trading_system
            .with_ctx_read(|ctx| ctx.get_history_positions().clone())
            .await;
        history_positions
    }

    pub async fn get_transactions(&self) -> Vec<VirtualTransaction> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let transactions = virtual_trading_system.with_ctx_read(|ctx| ctx.get_transactions().clone()).await;
        transactions
    }

    pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
        // let strategy_stats = self.strategy_stats.read().await;
        // strategy_stats.get_stats_history(play_index).await
        vec![]
    }

    pub async fn get_strategy_data(
        &self,
        datetime: Option<DateTime<Utc>>,
        index: Option<u64>,
        key: Key,
        limit: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, BacktestStrategyError> {
        // 安全检查：验证key是否属于当前策略
        let keys_map = self.keys.read().await;
        if !keys_map.contains_key(&key) {
            return Err(GetDataFailedSnafu {
                strategy_name: self.strategy_name().clone(),
                key: key.get_key_str(),
                datetime: datetime.map(|dt| dt.to_string()),
            }
            .fail()?);
        }
        drop(keys_map); // 释放锁

        match key {
            Key::Kline(kline_key) => {
                let kline_data = self.get_kline_slice(datetime, index, &kline_key, limit).await?;
                let kline_data = kline_data.0.iter().map(|kline| kline.to_json()).collect();
                Ok(kline_data)
            }
            Key::Indicator(indicator_key) => {
                let indicator_data = self.get_indicator_slice(datetime, index, &indicator_key, limit).await?;
                let indicator_data = indicator_data.0.iter().map(|indicator| indicator.to_json()).collect();
                Ok(indicator_data)
            }
        }
    }

    pub(super) async fn clear_data(&mut self) {
        let mut kline_data_guard = self.kline_data.write().await;
        kline_data_guard.iter_mut().for_each(|(key, kline_data)| {
            // 如果key不在min_interval_symbols中，则清空数据
            if key.interval() != self.min_interval {
                kline_data.clear();
            }
        });
        let mut indicator_data_guard = self.indicator_data.write().await;
        indicator_data_guard.iter_mut().for_each(|(key, indicator_data)| {
            // 如果kline_key不在min_interval_symbols中，则清空数据
            if key.interval() != self.min_interval {
                indicator_data.clear();
            }
        });
    }
}
