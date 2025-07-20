use super::CalculateIndicatorFunction;
use types::indicator::indicator::*;
use types::cache::CacheValue;
use std::sync::Arc;
use crate::indicator_engine::talib::TALib;
use crate::indicator_engine::talib_error::TalibError;
use types::indicator::PriceSource;




impl CalculateIndicatorFunction {
    pub async fn calculate_macd(macd_config: &MACDConfig, kline_series: Vec<Arc<CacheValue>>) -> Result<Vec<MACD>, String> {
        let timestamp_list: Vec<i64> = kline_series
            .iter()
            .enumerate()
            .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| kline.timestamp))
            .collect::<Result<Vec<_>, _>>()?;

        let price_source = match macd_config.price_source {
            PriceSource::Close => {
                kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {} for close price", i)).map(|kline| kline.close))
                    .collect::<Result<Vec<_>, _>>()?
            },
            PriceSource::Open => {
                kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {} for open price", i)).map(|kline| kline.open))
                    .collect::<Result<Vec<_>, _>>()?
            },
            PriceSource::High => {
                kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {} for high price", i)).map(|kline| kline.high))
                    .collect::<Result<Vec<_>, _>>()?
            },
            PriceSource::Low => {
                kline_series
                    .iter()
                    .enumerate()
                    .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {} for low price", i)).map(|kline| kline.low))
                    .collect::<Result<Vec<_>, _>>()?
            },
        };

        if price_source.len() < macd_config.fast_period as usize {
            return Ok(Vec::new());
        }

        let macd_result = match TALib::moving_average_convergence_divergence(
            &price_source, 
            macd_config.fast_period, 
            macd_config.slow_period, 
            macd_config.signal_period
        ) {
            Ok(result) => result,
            Err(e) => return Err(e.to_string()),
        };

        tracing::debug!("macd_result length: {}", macd_result.len());

        let mut macd_list = Vec::with_capacity(timestamp_list.len());

        // 使用 TA-Lib 的 MACD_Lookback 来计算正确的偏移量
        let lookback = unsafe { crate::indicator_engine::talib_bindings::TA_MACD_Lookback(
            macd_config.fast_period, 
            macd_config.slow_period, 
            macd_config.signal_period
        ) } as usize;
        
        tracing::debug!("MACD lookback: {}", lookback);

        for i in 0..timestamp_list.len() {
            let (macd_value, signal_value, histogram_value) = if i < lookback {
                // 在 lookback 期之前，所有值都是 NaN
                (f64::NAN, f64::NAN, f64::NAN)
            } else {
                let result_index = i - lookback;
                if result_index < macd_result.len() {
                    let values = &macd_result[result_index];
                    (values[0], values[1], values[2])  // [macd, signal, histogram]
                } else {
                    (f64::NAN, f64::NAN, f64::NAN)
                }
            };

            macd_list.push(MACD {
                timestamp: timestamp_list[i],
                macd: macd_value,
                signal: signal_value,
                histogram: histogram_value,
            });
        }

        Ok(macd_list)
    }
}
