// workspace crate
use star_river_core::{
    custom_type::PlayIndex,
    key::Key,
    market::QuantData,
    order::virtual_order::VirtualOrder,
    position::virtual_position::VirtualPosition,
    strategy::{StrategyVariable, strategy_benchmark::StrategyPerformanceReport},
    strategy_stats::StatsSnapshot,
    system::DateTimeUtc,
    transaction::virtual_transaction::VirtualTransaction,
};

// current crate
use super::BacktestStrategyContext;
use crate::error::strategy_error::{
    BacktestStrategyError, GetDataByDatetimeFailedSnafu, GetDataFailedSnafu, KlineDataLengthNotSameSnafu,
};


impl BacktestStrategyContext {
    // 获取所有的virtual order
    pub async fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let virtual_orders = virtual_trading_system.get_orders().clone();
        virtual_orders
    }

    pub async fn get_current_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let current_positions = virtual_trading_system.get_current_positions();
        current_positions.clone()
    }

    pub async fn get_history_positions(&self) -> Vec<VirtualPosition> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let history_positions = virtual_trading_system.get_history_positions();
        history_positions
    }

    pub async fn get_transactions(&self) -> Vec<VirtualTransaction> {
        let virtual_trading_system = self.virtual_trading_system.lock().await;
        let transactions = virtual_trading_system.get_transactions();
        transactions
    }

    pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
        let strategy_stats = self.strategy_stats.read().await;
        strategy_stats.get_stats_history(play_index).await
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
                    strategy_name: self.strategy_name.clone(),
                }
                .build());
            }
        }
    }

    pub async fn get_strategy_data(
        &self,
        play_index: PlayIndex,
        key: Key,
        limit: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, BacktestStrategyError> {
        // 安全检查：验证key是否属于当前策略
        let keys_map = self.keys.read().await;
        if !keys_map.contains_key(&key) {
            return Err(GetDataFailedSnafu {
                strategy_name: self.strategy_name.clone(),
                key: key.get_key_str(),
                play_index: play_index as u32,
            }
            .fail()?);
        }
        drop(keys_map); // 释放锁

        let index = match &key {
            Key::Kline(kline_key) => {
                if self.min_interval_symbols.contains(kline_key) {
                    if play_index == -1 { Some(0) } else { Some(play_index) }
                } else {
                    None
                }
            }
            Key::Indicator(indicator_key) => {
                let kline_key = indicator_key.get_kline_key();
                if self.min_interval_symbols.contains(&kline_key) {
                    if play_index == -1 { Some(0) } else { Some(play_index) }
                } else {
                    None
                }
            }
        };

        match key {
            Key::Kline(kline_key) => {
                let kline_data = self.get_kline_slice(&kline_key, index, limit).await?;
                let kline_data = kline_data.iter().map(|kline| kline.to_json()).collect();
                Ok(kline_data)
            }
            Key::Indicator(indicator_key) => {
                let indicator_data = self.get_indicator_slice(&indicator_key, index, None).await;
                let indicator_data = indicator_data.iter().map(|indicator| indicator.to_json()).collect();
                Ok(indicator_data)
            }
        }
    }

    pub async fn get_strategy_data_by_datetime(
        &self,
        key: Key,
        datetime: DateTimeUtc,
        limit: Option<i32>,
    ) -> Result<Vec<serde_json::Value>, BacktestStrategyError> {
        // 安全检查：验证key是否属于当前策略
        let keys_map = self.keys.read().await;
        if !keys_map.contains_key(&key) {
            return Err(GetDataByDatetimeFailedSnafu {
                strategy_name: self.strategy_name.clone(),
                key: key.get_key_str(),
                datetime: datetime.to_string(),
            }
            .fail()?);
        }
        drop(keys_map); // 释放锁

        match key {
            Key::Kline(kline_key) => {
                let kline_data_map_guard = self.kline_data.read().await;
                let play_index = kline_data_map_guard
                    .get(&kline_key)
                    .unwrap()
                    .iter()
                    .position(|kline| kline.datetime() == datetime)
                    .unwrap();
                let kline_data = self.get_kline_slice(&kline_key, Some(play_index as i32), limit).await?;
                let kline_data = kline_data.iter().map(|kline| kline.to_json()).collect();
                Ok(kline_data)
            }
            Key::Indicator(indicator_key) => {
                let indicator_data_map_guard = self.indicator_data.read().await;
                let play_index = indicator_data_map_guard
                    .get(&indicator_key)
                    .unwrap()
                    .iter()
                    .position(|indicator| indicator.get_datetime() == datetime)
                    .unwrap();
                let indicator_data = self.get_indicator_slice(&indicator_key, Some(play_index as i32), limit).await;
                let indicator_data = indicator_data.iter().map(|indicator| indicator.to_json()).collect();
                Ok(indicator_data)
            }
        }
    }

    pub(super) async fn clear_data(&mut self) {
        let mut kline_data_guard = self.kline_data.write().await;
        kline_data_guard.iter_mut().for_each(|(key, kline_data)| {
            // 如果key不在min_interval_symbols中，则清空数据
            if !self.min_interval_symbols.contains(key) {
                kline_data.clear();
            }
        });
        let mut indicator_data_guard = self.indicator_data.write().await;
        indicator_data_guard.iter_mut().for_each(|(key, indicator_data)| {
            let kline_key = key.get_kline_key();
            // 如果kline_key不在min_interval_symbols中，则清空数据
            if !self.min_interval_symbols.contains(&kline_key) {
                indicator_data.clear();
            }
        });
    }

    pub(super) async fn reset_all_custom_variables(&mut self) {
        let mut custom_variable_guard = self.custom_variable.write().await;
        custom_variable_guard.iter_mut().for_each(|(_, custom_variable)| {
            custom_variable.var_value = custom_variable.initial_value.clone();
        });
    }

    pub(super) async fn reset_all_sys_variables(&mut self) {
        let mut sys_variable_guard = self.sys_variable.write().await;
        sys_variable_guard.clear();
    }

    pub async fn get_strategy_variable(&self) -> Vec<StrategyVariable> {
        let mut strategy_variable = Vec::new();

        let custom_variable_guard = self.custom_variable.read().await;
        let custom_var = custom_variable_guard
            .iter()
            .map(|(_, custom_variable)| StrategyVariable::CustomVariable(custom_variable.clone()))
            .collect::<Vec<StrategyVariable>>();

        let sys_variable_guard = self.sys_variable.read().await;
        let sys_var = sys_variable_guard
            .iter()
            .map(|(_, sys_variable)| StrategyVariable::SysVariable(sys_variable.clone()))
            .collect::<Vec<StrategyVariable>>();

        strategy_variable.extend(custom_var);
        strategy_variable.extend(sys_var);
        strategy_variable
    }

    pub async fn get_strategy_performance_report(&self) -> StrategyPerformanceReport {
        let strategy_benchmark_guard = self.benchmark.read().await;
        strategy_benchmark_guard.report().clone()
    }
}
