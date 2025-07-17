pub mod sma;
pub mod macd;
pub mod bbands;

use tokio::sync::Mutex;
use crate::cache_engine::CacheEngine;
use types::indicator::Indicator;
use std::sync::Arc;
use types::cache::Key;
use types::indicator::IndicatorConfig;
use types::cache::CacheValue;
use crate::indicator_engine::talib::TALib;
use types::indicator::sma::SMA;
use types::indicator::macd::MACD;
use types::indicator::bbands::BBands;
use types::indicator::PriceSource;



pub struct CalculateIndicatorFunction;



impl CalculateIndicatorFunction {

    pub async fn calculate_indicator(
        cache_engine: Arc<Mutex<CacheEngine>>, 
        kline_key: Key,
        indicator_config: IndicatorConfig,
        ignore_config: bool // 是否忽略指标计算配置中所需要的长度，而是使用缓存中的所有数据
    ) -> Result<Vec<Indicator>, String> {
        tracing::info!("indicator_config: {:?}", indicator_config);

        let lookback = TALib::lookback(&indicator_config);
        let kline_series: Vec<Arc<CacheValue>>;
        
        if ignore_config  {
            kline_series = cache_engine.lock().await.get_cache_value(&kline_key, None, None).await;
        } else {
            kline_series = cache_engine.lock().await.get_cache_value(&kline_key, None,Some(lookback+1)).await;
            if kline_series.len() < (lookback+1) as usize {
                return Err(format!("kline_series length is less than lookback: {:?}", lookback));
            }
        }

        match &indicator_config {
            IndicatorConfig::SMA(sma_config) => {
                let sma_list: Vec<SMA> = CalculateIndicatorFunction::calculate_sma(sma_config, kline_series).await.unwrap();
                let sma: Vec<Indicator> = sma_list.into_iter().map(|sma| sma.into()).collect();
                Ok(sma)
            }
            IndicatorConfig::MACD(macd_config) => {
                tracing::info!("macd lookback: {:?}", lookback);

                let macd_list: Vec<MACD> = CalculateIndicatorFunction::calculate_macd(macd_config, kline_series).await.unwrap();
                let macd: Vec<Indicator> = macd_list.into_iter().map(|macd| macd.into()).collect();
                Ok(macd)
            }
            IndicatorConfig::BBands(bbands_config) => {
                let bbands_list: Vec<BBands> = CalculateIndicatorFunction::calculate_bbands(bbands_config, kline_series, lookback).await.unwrap();
                let bbands: Vec<Indicator> = bbands_list.into_iter().map(|bbands| bbands.into()).collect();
                Ok(bbands)
            }
        }
    }


    fn get_price_source(price_source: &PriceSource, kline_series: Vec<Arc<CacheValue>>) -> Vec<f64> {
        match price_source {
            PriceSource::Close => kline_series.iter().map(|v| v.as_kline().unwrap().close).collect(),
            PriceSource::Open => kline_series.iter().map(|v| v.as_kline().unwrap().open).collect(),
            PriceSource::High => kline_series.iter().map(|v| v.as_kline().unwrap().high).collect(),
            PriceSource::Low => kline_series.iter().map(|v| v.as_kline().unwrap().low).collect(),
        }
    }
}
