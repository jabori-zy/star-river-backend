use super::CalculateIndicatorFunction;
use types::indicator::indicator::*;
use types::cache::CacheValue;
use std::sync::Arc;
use crate::indicator_engine::talib::TALib;
use crate::indicator_engine::talib_error::TalibError;
use types::indicator::PriceSource;



impl CalculateIndicatorFunction {
    pub async fn calculate_bbands(bbands_config: &BBandsConfig, kline_series: Vec<Arc<CacheValue>>, lookback: u32) -> Result<Vec<BBands>, String> {
        let timestamp_list: Vec<i64> = kline_series.iter().map(|v| v.as_kline().unwrap().timestamp).collect();
        
        let price_source = CalculateIndicatorFunction::get_price_source_and_timestamp(&bbands_config.price_source, kline_series);

        let bbands_result = match TALib::bollinger_bands(
            &price_source, 
            bbands_config.period, 
            bbands_config.dev_up.into_inner(), 
            bbands_config.dev_down.into_inner(), 
            bbands_config.ma_type.clone() as i32
        ) {
            Ok(result) => result,
            Err(e) => return Err(e.to_string()),
        };

        let mut bbands_list = Vec::with_capacity(timestamp_list.len());

        for i in 0..timestamp_list.len() {
            let (upper, middle, lower) = if i < lookback as usize {
                (f64::NAN, f64::NAN, f64::NAN)
            } else {
                let result_index = i - lookback as usize;
                if result_index < bbands_result.len() {
                    let values = &bbands_result[result_index];
                    (values[0], values[1], values[2])
                } else {
                    (f64::NAN, f64::NAN, f64::NAN)
                }
            };

            bbands_list.push(BBands {
                timestamp: timestamp_list[i],
                upper,
                middle,
                lower,
            });
        }
        Ok(bbands_list)
    }
}