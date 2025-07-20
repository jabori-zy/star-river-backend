// pub mod sma;
// pub mod macd;
// pub mod bbands;
pub mod calculate_macros;

use tokio::sync::Mutex;
use crate::cache_engine::CacheEngine;
use types::indicator::Indicator;
use std::sync::Arc;
use types::cache::Key;
use types::indicator::IndicatorConfig;
use types::cache::CacheValue;
use crate::indicator_engine::talib::TALib;
use types::indicator::indicator::*;
use types::indicator::PriceSource;
use crate::calculate_fn;



pub struct CalculateIndicatorFunction;



impl CalculateIndicatorFunction {

    pub async fn calculate_indicator(
        cache_engine: Arc<Mutex<CacheEngine>>, 
        kline_key: Key,
        indicator_config: IndicatorConfig,
        ignore_config: bool // 是否忽略指标计算配置中所需要的长度，而是使用缓存中的所有数据
    ) -> Result<Vec<Indicator>, String> {
        tracing::info!("indicator_config: {:?}", indicator_config);

        // let lookback = TALib::lookback(&indicator_config);
        // let kline_series: Vec<Arc<CacheValue>>;
        
        // if ignore_config  {
        //     kline_series = cache_engine.lock().await.get_cache_value(&kline_key, None, None).await;
        // } else {
        //     kline_series = cache_engine.lock().await.get_cache_value(&kline_key, None,Some(lookback+1)).await;
        //     if kline_series.len() < (lookback+1) as usize {
        //         return Err(format!("kline_series length is less than lookback: {:?}", lookback));
        //     }
        // }

        // match &indicator_config {
        //     IndicatorConfig::SMA(sma_config) => {
        //         let sma_list: Vec<SMA> = CalculateIndicatorFunction::calculate_sma(sma_config, kline_series).await.unwrap();
        //         let sma: Vec<Indicator> = sma_list.into_iter().map(|sma| sma.into()).collect();
        //         Ok(sma)
        //     }
        //     IndicatorConfig::MACD(macd_config) => {
        //         tracing::info!("macd lookback: {:?}", lookback);

        //         let macd_list: Vec<MACD> = CalculateIndicatorFunction::calculate_macd(macd_config, kline_series).await.unwrap();
        //         let macd: Vec<Indicator> = macd_list.into_iter().map(|macd| macd.into()).collect();
        //         Ok(macd)
        //     }
        //     IndicatorConfig::BBands(bbands_config) => {
        //         let bbands_list: Vec<BBands> = CalculateIndicatorFunction::calculate_bbands(bbands_config, kline_series, lookback).await.unwrap();
        //         let bbands: Vec<Indicator> = bbands_list.into_iter().map(|bbands| bbands.into()).collect();
        //         Ok(bbands)
        //     }
        //     _ => {
        //         return Err(format!("indicator_config: {:?} not supported", indicator_config));
        //     }
        // }
        Ok(Vec::new())
    }


    fn get_price_source_and_timestamp(price_source: &PriceSource, kline_series: Vec<Arc<CacheValue>>) -> Result<(Vec<i64>, Vec<f64>), String> {
        let (timestamp_list, price_list) = match price_source {
            PriceSource::Close => {
                let (timestamp_list, close_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.close)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, close_list)
            },
            PriceSource::Open => {
                let (timestamp_list, open_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.open)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, open_list)
            },
            PriceSource::High => {
                let (timestamp_list, high_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.high)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, high_list)
            },
            PriceSource::Low => {
                let (timestamp_list, low_list): (Vec<i64>, Vec<f64>) = kline_series
                .iter()
                .enumerate()
                .map(|(i, v)| v.as_kline().ok_or_else(|| format!("Invalid kline data at index {}", i)).map(|kline| (kline.timestamp, kline.low)))
                .collect::<Result<Vec<(i64, f64)>, _>>()?
                .into_iter()
                .unzip();
                (timestamp_list, low_list)
            },
        };

        Ok((timestamp_list, price_list))
    }


    calculate_fn!(MA,
        talib_params => [
            (time_period: i32),
            (ma_type: MAType),
        ],
    );

    calculate_fn!(MACD,
        talib_params => [
            (fast_period: i32),
            (slow_period: i32),
            (signal_period: i32),
        ],
    );

    calculate_fn!(BBands,
        talib_params => [
            (time_period: i32),
            (dev_up: f64),
            (dev_down: f64),
            (ma_type: MAType),
        ],
    );


}
