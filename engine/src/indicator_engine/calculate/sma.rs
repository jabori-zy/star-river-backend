use super::CalculateIndicatorFunction;
use types::indicator::sma::{SMA, SMAConfig};
use types::cache::CacheValue;
use std::sync::Arc;
use crate::indicator_engine::talib::TALib;
use crate::indicator_engine::talib_error::TalibError;

impl CalculateIndicatorFunction {

    pub async fn calculate_sma(sma_config: SMAConfig, kline_series: Vec<Arc<CacheValue>>) -> Result<Vec<SMA>, String> {
        // 提取时间戳和收盘价
        let timestamp_list: Vec<i64> = kline_series.iter().map(|v| v.as_kline().unwrap().timestamp).collect(); 
        let close: Vec<f64> = kline_series.iter().map(|v| v.as_kline().unwrap().close).collect();

        // 使用TALib::sma计算SMA值
        let sma_result = match TALib::sma(&close, sma_config.period) {
            Ok(values) => values,
            Err(e) => return Err(e.to_string()),
        };

        // 计算有效起始索引
        let offset = (sma_config.period - 1) as usize;
        
        // 如果没有足够的数据计算SMA，返回空结果
        if offset >= timestamp_list.len() || sma_result.is_empty() {
            return Ok(Vec::new());
        }

        // 创建只包含有效结果的SMA列表
        let mut sma_list = Vec::with_capacity(sma_result.len());
        
        // 只使用有效的时间戳和SMA值
        for i in 0..sma_result.len() {
            if offset + i < timestamp_list.len() {
                sma_list.push(SMA {
                    timestamp: timestamp_list[offset + i],
                    sma: sma_result[i]
                });
            }
        }
        
        Ok(sma_list)
    }

    pub async fn calculate_sma1(sma_config: SMAConfig, kline_series: Vec<Arc<CacheValue>>) -> Result<Vec<SMA>, String> {
        // 提取时间戳和收盘价
        let timestamp_list: Vec<i64> = kline_series.iter().map(|v| v.as_kline().unwrap().timestamp).collect(); 
        let close: Vec<f64> = kline_series.iter().map(|v| v.as_kline().unwrap().close).collect();

        // 如果没有足够的数据，返回空结果
        if close.len() < sma_config.period as usize {
            return Ok(Vec::new());
        }

        // 使用TALib::sma计算SMA值
        let sma_result = match TALib::sma(&close, sma_config.period) {
            Ok(values) => values,
            Err(e) => return Err(e.to_string()),
        };

        // 创建包含所有结果的SMA列表
        let mut sma_list = Vec::with_capacity(timestamp_list.len());
        
        // 计算偏移量（前period-1个数据点没有有效的SMA值）
        let offset = (sma_config.period - 1) as usize;
        
        // 处理所有时间戳
        for i in 0..timestamp_list.len() {
            let sma_value = if i < offset {
                // 前period-1个数据点使用NaN
                f64::NAN
            } else if i - offset < sma_result.len() {
                // 有效的SMA值
                sma_result[i - offset]
            } else {
                // 超出范围，使用NaN
                f64::NAN
            };
            
            sma_list.push(SMA {
                timestamp: timestamp_list[i],
                sma: sma_value
            });
        }
        
        Ok(sma_list)
    }
}
