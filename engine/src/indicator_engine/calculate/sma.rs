use super::CalculateIndicatorFunction;
use types::indicator::indicator::*;
use types::cache::CacheValue;
use std::sync::Arc;
use crate::indicator_engine::talib::TALib;
use crate::indicator_engine::talib_error::TalibError;

impl CalculateIndicatorFunction {

    pub async fn calculate_sma(ma_config: &MAConfig, kline_series: Vec<Arc<CacheValue>>) -> Result<Vec<MA>, String> {
        // 提取时间戳和收盘价
        let timestamp_list: Vec<i64> = kline_series.iter().map(|v| v.as_kline().unwrap().timestamp).collect(); 
        let close: Vec<f64> = kline_series.iter().map(|v| v.as_kline().unwrap().close).collect();

        // 如果没有足够的数据，返回空结果
        if close.len() < ma_config.period as usize {
            return Ok(Vec::new());
        }

        // 使用TALib::sma计算SMA值
        let sma_result = match TALib::simple_moving_average(&close, ma_config.period) {
            Ok(values) => values,
            Err(e) => return Err(e.to_string()),
        };

        // 创建包含所有结果的SMA列表
        let mut ma_list = Vec::with_capacity(timestamp_list.len());
        
        // 计算偏移量（前period-1个数据点没有有效的SMA值）
        let offset = (ma_config.period - 1) as usize;
        
        // 处理所有时间戳
        for i in 0..timestamp_list.len() {
            let ma_value = if i < offset {
                // 前period-1个数据点使用NaN
                f64::NAN
            } else if i - offset < sma_result.len() {
                // 有效的SMA值
                sma_result[i - offset]
            } else {
                // 超出范围，使用NaN
                f64::NAN
            };
            
            ma_list.push(MA {
                timestamp: timestamp_list[i],
                ma: ma_value
            });
        }
        
        Ok(ma_list)
    }
}
