use chrono::{DateTime, Utc};
// workspace crate
use key::{Key, KeyTrait};
use star_river_core::system::DateTimeUtc;
use strategy_core::strategy::context_trait::StrategyIdentityExt;
use strategy_stats::StatsSnapshot;
use virtual_trading::types::{VirtualOrder, VirtualPosition, VirtualTransaction};

// current crate
use super::BacktestStrategyContext;
use crate::strategy::{
    PlayIndex,
    strategy_error::{BacktestStrategyError, GetDataByDatetimeFailedSnafu, GetDataFailedSnafu, KlineDataLengthNotSameSnafu},
};

impl BacktestStrategyContext {
    // 获取所有的virtual order
    pub async fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        // let virtual_trading_system = self.virtual_trading_system.lock().await;
        // let virtual_orders = virtual_trading_system.get_orders().clone();
        // virtual_orders
        vec![]
    }

    pub async fn get_current_positions(&self) -> Vec<VirtualPosition> {
        // let virtual_trading_system = self.virtual_trading_system.lock().await;
        // let current_positions = virtual_trading_system.get_current_positions();
        // current_positions.clone()
        vec![]
    }

    pub async fn get_history_positions(&self) -> Vec<VirtualPosition> {
        // let virtual_trading_system = self.virtual_trading_system.lock().await;
        // let history_positions = virtual_trading_system.get_history_positions();
        // history_positions
        vec![]
    }

    pub async fn get_transactions(&self) -> Vec<VirtualTransaction> {
        // let virtual_trading_system = self.virtual_trading_system.lock().await;
        // let transactions = virtual_trading_system.get_transactions();
        // transactions
        vec![]
    }

    pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
        // let strategy_stats = self.strategy_stats.read().await;
        // strategy_stats.get_stats_history(play_index).await
        vec![]
    }

    pub async fn get_signal_count(&self) -> Result<i32, BacktestStrategyError> {
        let kline_data_guard = self.kline_data.read().await;
        // 获取每一个key对应列表的长度
        let kline_data_lengths = kline_data_guard.values().map(|data| data.len() as i32).collect::<Vec<i32>>();

        // 如果只有一个key，则返回该key的长度
        if kline_data_lengths.len() == 1 {
            return Ok(kline_data_lengths[0]);
        } else {
            // 如果有多个key，比较列表中的长度是否全部相同
            if kline_data_lengths.iter().all(|length| length == &kline_data_lengths[0]) {
                return Ok(kline_data_lengths[0]);
            } else {
                return Err(KlineDataLengthNotSameSnafu {
                    strategy_name: self.strategy_name().clone(),
                }
                .build());
            }
        }
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
            _ => Ok(vec![]), // Key::Indicator(indicator_key) => {
                             //     let indicator_data = self.get_indicator_slice(&indicator_key, index, None).await;
                             //     let indicator_data = indicator_data.iter().map(|indicator| indicator.to_json()).collect();
                             //     Ok(indicator_data)
                             // }
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
