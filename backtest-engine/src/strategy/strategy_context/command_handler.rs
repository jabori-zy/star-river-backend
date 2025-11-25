// workspace crate
use chrono::{DateTime, Utc};
use key::{IndicatorKey, KeyTrait, KlineKey};
use star_river_core::kline::Kline;
use strategy_core::strategy::context_trait::StrategyIdentityExt;
use ta_lib::Indicator;

// current crate
use super::BacktestStrategyContext;
use crate::strategy::strategy_error::{BacktestStrategyError, KeyNotFoundSnafu};

/// Generic helper method to slice time-series data with datetime-based indexing
/// Supports fast-path optimization via index hint and binary search fallback
fn slice_time_series_data<T, F>(
    data: &[T],
    datetime: Option<DateTime<Utc>>,
    index: Option<u64>,
    limit: Option<i32>,
    get_datetime: F,
) -> Result<(Vec<T>, Option<u64>), BacktestStrategyError>
where
    T: Clone,
    F: Fn(&T) -> DateTime<Utc>,
{
    let data_length = data.len();

    match (datetime, limit) {
        // Both datetime and limit are provided
        (Some(datetime), Some(limit)) => {
            // Try fast path first if index hint is provided
            let target_index = if let Some(hint_index) = index {
                let hint_idx = hint_index as usize;
                // Fast path: O(1) check if index hint matches datetime
                if hint_idx < data_length && get_datetime(&data[hint_idx]) == datetime {
                    // ⚡ Fast path hit! Index matches datetime
                    hint_idx
                } else {
                    // Index hint didn't match, fallback to binary search
                    let search_result = data.binary_search_by(|item| get_datetime(item).cmp(&datetime));
                    match search_result {
                        Ok(exact_index) => exact_index,
                        Err(insert_pos) => {
                            if insert_pos == 0 {
                                return Ok((Vec::new(), None));
                            }
                            insert_pos - 1
                        }
                    }
                }
            } else {
                // No index hint, use binary search directly
                let search_result = data.binary_search_by(|item| get_datetime(item).cmp(&datetime));
                match search_result {
                    Ok(exact_index) => exact_index,
                    Err(insert_pos) => {
                        if insert_pos == 0 {
                            return Ok((Vec::new(), None));
                        }
                        insert_pos - 1
                    }
                }
            };

            // Calculate slice: take `limit` items up to and including target_index
            let end = target_index + 1;
            let start = if limit as usize >= end { 0 } else { end - limit as usize };
            Ok((data[start..end].to_vec(), Some(target_index as u64)))
        }
        // datetime provided, no limit
        (Some(datetime), None) => {
            // Try fast path first if index hint is provided
            let target_index = if let Some(hint_index) = index {
                let hint_idx = hint_index as usize;
                // Fast path: O(1) check if index hint matches datetime
                if hint_idx < data_length && get_datetime(&data[hint_idx]) == datetime {
                    // ⚡ Fast path hit! Index matches datetime
                    hint_idx
                } else {
                    // Index hint didn't match, fallback to binary search
                    let search_result = data.binary_search_by(|item| get_datetime(item).cmp(&datetime));
                    match search_result {
                        Ok(exact_index) => exact_index,
                        Err(insert_pos) => {
                            if insert_pos == 0 {
                                return Ok((Vec::new(), None));
                            }
                            insert_pos - 1
                        }
                    }
                }
            } else {
                // No index hint, use binary search directly
                let search_result = data.binary_search_by(|item| get_datetime(item).cmp(&datetime));
                match search_result {
                    Ok(exact_index) => exact_index,
                    Err(insert_pos) => {
                        if insert_pos == 0 {
                            return Ok((Vec::new(), None));
                        }
                        insert_pos - 1
                    }
                }
            };

            // Return all data from start to target_index (inclusive)
            let end = target_index + 1;
            Ok((data[0..end].to_vec(), Some(target_index as u64)))
        }
        // No datetime, but limit provided
        (None, Some(limit)) => {
            // Take last `limit` items from the data
            if limit as usize >= data_length {
                Ok((data.to_vec(), None))
            } else {
                let start = data_length.saturating_sub(limit as usize);
                Ok((data[start..].to_vec(), None))
            }
        }
        // Neither datetime nor limit provided
        (None, None) => {
            // Return all data
            Ok((data.to_vec(), None))
        }
    }
}

mod kline {
    use std::collections::hash_map::Entry;

    use chrono::{DateTime, Utc};
    use snafu::OptionExt;

    use super::*;
    use crate::strategy::strategy_error::SymbolIsNotMinIntervalSnafu;

    impl BacktestStrategyContext {
        pub async fn init_kline_data(&mut self, kline_key: &KlineKey, init_kline_data: Vec<Kline>) -> Result<(), BacktestStrategyError> {
            // check if kline_key is in min_interval_symbols
            if kline_key.interval() != self.min_interval {
                return Err(SymbolIsNotMinIntervalSnafu {
                    strategy_name: self.strategy_name().clone(),
                    symbol: kline_key.symbol().clone(),
                    interval: kline_key.interval().to_string(),
                }
                .build());
            };

            // init kline data
            let mut kline_data_guard = self.kline_data.write().await;
            match kline_data_guard.entry(kline_key.clone()) {
                Entry::Vacant(e) => {
                    e.insert(init_kline_data);
                }
                Entry::Occupied(mut e) => {
                    if e.get().is_empty() {
                        e.insert(init_kline_data);
                    }
                }
            }
            Ok(())
        }

        pub async fn append_kline_data(&mut self, kline_key: &KlineKey, kline_series: Vec<Kline>) -> Result<(), BacktestStrategyError> {
            // check if kline_key is in min_interval_symbols
            if kline_key.interval() != self.min_interval {
                return Err(SymbolIsNotMinIntervalSnafu {
                    strategy_name: self.strategy_name().clone(),
                    symbol: kline_key.symbol().clone(),
                    interval: kline_key.interval().to_string(),
                }
                .build());
            };

            let mut kline_data_guard = self.kline_data.write().await;
            match kline_data_guard.entry(kline_key.clone()) {
                Entry::Occupied(mut e) => {
                    let kline_data = e.get_mut();
                    kline_data.extend(kline_series);
                    // Sort by timestamp to ensure correct time order
                    kline_data.sort_by(|a, b| a.datetime().cmp(&b.datetime()));
                    // Deduplicate: remove duplicates with same datetime, keep the last one
                    kline_data.dedup_by(|a, b| a.datetime() == b.datetime());
                }
                Entry::Vacant(e) => {
                    // New data also needs sorting and deduplication
                    let mut sorted_series = kline_series;
                    sorted_series.sort_by(|a, b| a.datetime().cmp(&b.datetime()));
                    sorted_series.dedup_by(|a, b| a.datetime() == b.datetime());
                    e.insert(sorted_series);
                }
            }
            Ok(())
        }

        pub async fn get_kline_slice(
            &self,
            datetime: Option<DateTime<Utc>>,
            index: Option<u64>,
            kline_key: &KlineKey,
            limit: Option<i32>,
        ) -> Result<(Vec<Kline>, Option<u64>), BacktestStrategyError> {
            let kline_data_guard = self.kline_data.read().await;
            let data = kline_data_guard.get(kline_key).context(KeyNotFoundSnafu {
                strategy_name: self.strategy_name(),
                key: kline_key.key_str(),
            })?;

            // Use generic helper method with kline datetime extractor
            slice_time_series_data(data, datetime, index, limit, |k| k.datetime())
        }

        pub async fn update_kline_data(&mut self, kline_key: &KlineKey, kline: &Kline) -> Kline {
            // 先检查键是否存在，释放锁
            let key_exists = { self.kline_data.read().await.contains_key(kline_key) };

            if !key_exists {
                // 如果缓存键不存在，先初始化空的Vec
                let mut kline_data_guard = self.kline_data.write().await;
                kline_data_guard.insert(kline_key.clone(), Vec::new());
            }

            // 重新获取锁并更新
            let mut kline_data_guard = self.kline_data.write().await;
            let kline_data = kline_data_guard.get_mut(kline_key).unwrap();

            if !key_exists || kline_data.len() == 0 {
                // 判断是否为初始化
                kline_data.clear();
                kline_data.push(kline.clone());
            } else {
                // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k线
                if let Some(last_kline) = kline_data.last() {
                    if last_kline.datetime() == kline.datetime() {
                        kline_data.pop();
                        kline_data.push(kline.clone());
                    } else {
                        // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
                        kline_data.push(kline.clone());
                    }
                } else {
                    kline_data.push(kline.clone());
                }
            }

            kline.clone()
        }
    }
}

mod indicator {
    use chrono::{DateTime, Utc};
    use snafu::OptionExt;

    use super::*;

    impl BacktestStrategyContext {
        pub async fn init_indicator_data(&mut self, indicator_key: &IndicatorKey, indicator_series: Vec<Indicator>) {
            // 初始化指标数据
            let mut indicator_data_guard = self.indicator_data.write().await;
            // 如果指标key存在
            if let Some(indicator_data) = indicator_data_guard.get(indicator_key) {
                // 如果指标数据为空，则初始化指标数据
                if indicator_data.len() == 0 {
                    indicator_data_guard.insert(indicator_key.clone(), indicator_series);
                }
            } else {
                // 如果指标key不存在，则初始化指标数据
                indicator_data_guard.insert(indicator_key.clone(), indicator_series);
            }
        }

        pub async fn get_indicator_slice(
            &self,
            datetime: Option<DateTime<Utc>>,
            index: Option<u64>,
            indicator_key: &IndicatorKey,
            limit: Option<i32>,
        ) -> Result<(Vec<Indicator>, Option<u64>), BacktestStrategyError> {
            let indicator_data_guard = self.indicator_data.read().await;
            let data = indicator_data_guard.get(indicator_key).context(KeyNotFoundSnafu {
                strategy_name: self.strategy_name(),
                key: indicator_key.key_str(),
            })?;

            // Use generic helper method with indicator datetime extractor
            slice_time_series_data(data, datetime, index, limit, |ind| ind.get_datetime())
        }

        pub async fn update_indicator_data(&mut self, indicator_key: &IndicatorKey, indicator: &Indicator) -> Indicator {
            // 先检查键是否存在，释放锁
            let key_exists = { self.indicator_data.read().await.contains_key(indicator_key) };

            if !key_exists {
                // 如果缓存键不存在，先初始化空的Vec
                let mut indicator_data_guard = self.indicator_data.write().await;
                indicator_data_guard.insert(indicator_key.clone(), Vec::new());
            }

            // 重新获取锁并更新
            let mut indicator_data_guard = self.indicator_data.write().await;
            let indicator_data = indicator_data_guard.get_mut(indicator_key).unwrap();

            if !key_exists || indicator_data.len() == 0 {
                // 判断是否为初始化
                indicator_data.clear();
                indicator_data.push(indicator.clone());
            } else {
                // 如果最新的一条数据时间戳等于最后一个指标的时间戳，则更新最后一条指标
                if let Some(last_indicator) = indicator_data.last() {
                    if last_indicator.get_datetime() == indicator.get_datetime() {
                        indicator_data.pop();
                        indicator_data.push(indicator.clone());
                    } else {
                        // 如果最新的一条数据时间戳不等于最后一个指标的时间戳，则插入新数据
                        indicator_data.push(indicator.clone());
                    }
                } else {
                    indicator_data.push(indicator.clone());
                }
            }

            indicator.clone()
        }
    }
}
