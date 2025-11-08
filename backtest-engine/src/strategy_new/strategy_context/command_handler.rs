// workspace crate
use ta_lib::Indicator;

// current crate
use super::BacktestStrategyContext;
use crate::{
    error::strategy_error::{BacktestStrategyError, KlineKeyNotFoundSnafu, PlayIndexOutOfRangeSnafu},
};
use key::{KlineKey, IndicatorKey};
use star_river_core::kline::Kline;
use strategy_core::strategy::context_trait::{StrategyIdentityExt};
use key::KeyTrait;


mod kline {

    use super::*;

    impl BacktestStrategyContext {
        pub async fn init_kline_data(&mut self, kline_key: &KlineKey, init_kline_data: Vec<Kline>) {
            // 初始化k线数据
            let mut kline_data_guard = self.kline_data.write().await;
            if let Some(kline_data) = kline_data_guard.get(kline_key) {
                if kline_data.len() == 0 {
                    kline_data_guard.insert(kline_key.clone(), init_kline_data);
                }
            } else {
                kline_data_guard.insert(kline_key.clone(), init_kline_data);
            }
        }

        pub async fn append_kline_data(&mut self, kline_key: &KlineKey, kline_series: Vec<Kline>) {
            let mut kline_data_guard = self.kline_data.write().await;
            if let Some(kline_data) = kline_data_guard.get_mut(kline_key) {
                kline_data.extend(kline_series);
                // 按时间戳排序，确保K线数据的时序正确性
                kline_data.sort_by(|a, b| a.datetime().cmp(&b.datetime()));
                // 去重：移除相同datetime的重复数据，保留最后一个
                kline_data.dedup_by(|a, b| a.datetime() == b.datetime());
            } else {
                // 新插入的数据也需要排序和去重
                let mut sorted_series = kline_series;
                sorted_series.sort_by(|a, b| a.datetime().cmp(&b.datetime()));
                sorted_series.dedup_by(|a, b| a.datetime() == b.datetime());
                kline_data_guard.insert(kline_key.clone(), sorted_series);
            }
        }

        pub async fn get_kline_slice(
            &self,
            kline_key: &KlineKey,
            play_index: Option<i32>,
            limit: Option<i32>,
        ) -> Result<Vec<Kline>, BacktestStrategyError> {
            let kline_data = self.get_kline_data(kline_key).await;
            if let Some(data) = kline_data {
                let kline_data_length = data.len() as u32;
                match (play_index, limit) {
                    // 有index，有limit
                    (Some(play_index), Some(limit)) => {
                        // 如果索引超出范围，返回空
                        if play_index as u32 >= kline_data_length {
                            Err(PlayIndexOutOfRangeSnafu {
                                strategy_name: self.strategy_name().clone(),
                                kline_data_length: kline_data_length,
                                play_index: play_index as u32,
                            }
                            .build())
                        } else {
                            // 计算从索引开始向前取limit个元素
                            let end = play_index as usize + 1;
                            let start = if limit as usize >= end { 0 } else { end - limit as usize };
                            Ok(data[start..end].to_vec())
                        }
                    }
                    // 有index，无limit
                    (Some(play_index), None) => {
                        // 如果索引超出范围，返回空
                        if play_index as u32 >= kline_data_length {
                            Err(PlayIndexOutOfRangeSnafu {
                                strategy_name: self.strategy_name().clone(),
                                kline_data_length: kline_data_length,
                                play_index: play_index as u32,
                            }
                            .build())
                        } else {
                            // 从索引开始向前取所有元素（到开头）
                            let end = play_index as usize + 1;
                            Ok(data[0..end].to_vec())
                        }
                    }
                    // 无index，有limit
                    (None, Some(limit)) => {
                        // 从后往前取limit条数据
                        if limit as u32 >= kline_data_length {
                            Ok(data.clone())
                        } else {
                            let start = (kline_data_length as usize).saturating_sub(limit as usize);
                            Ok(data[start..].to_vec())
                        }
                    }
                    // 无index，无limit
                    (None, None) => {
                        // 如果limit和index都为None，则返回所有数据
                        Ok(data.clone())
                    }
                }
            } else {
                Err(KlineKeyNotFoundSnafu {
                    strategy_name: self.strategy_name().clone(),
                    kline_key: kline_key.get_key_str(),
                }
                .build())
            }
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
            indicator_key: &IndicatorKey,
            play_index: Option<i32>,
            limit: Option<i32>,
        ) -> Vec<Indicator> {
            let indicator_data_guard = self.indicator_data.read().await;
            if let Some(indicator_data) = indicator_data_guard.get(indicator_key) {
                match (play_index, limit) {
                    // 有index，有limit
                    (Some(play_index), Some(limit)) => {
                        // 如果索引超出范围，返回空Vec
                        if play_index as usize >= indicator_data.len() {
                            Vec::new()
                        } else {
                            // 计算从索引开始向前取limit个元素
                            let end = play_index as usize + 1;
                            let start = if limit as usize >= end { 0 } else { end - limit as usize };
                            indicator_data[start..end].to_vec()
                        }
                    }
                    // 有index，无limit
                    (Some(play_index), None) => {
                        // 如果索引超出范围，返回空Vec
                        if play_index as usize >= indicator_data.len() {
                            Vec::new()
                        } else {
                            // 从索引开始向前取所有元素（到开头）
                            let end = play_index as usize + 1;
                            indicator_data[0..end].to_vec()
                        }
                    }
                    // 无index，有limit
                    (None, Some(limit)) => {
                        // 从后往前取limit条数据
                        if limit as usize >= indicator_data.len() {
                            indicator_data.clone()
                        } else {
                            let start = indicator_data.len().saturating_sub(limit as usize);
                            indicator_data[start..].to_vec()
                        }
                    }
                    // 无index，无limit
                    (None, None) => {
                        // 如果limit和index都为None，则返回所有数据
                        indicator_data.clone()
                    }
                }
            } else {
                Vec::new()
            }
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