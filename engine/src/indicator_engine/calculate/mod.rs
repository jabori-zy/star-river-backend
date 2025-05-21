pub mod sma;


use tokio::sync::Mutex;
use crate::cache_engine::CacheEngine;
use types::indicator::Indicator;
use std::sync::Arc;
use crate::indicator_engine::indicator_engine_type::IndicatorSubKey;
use types::cache::cache_key::KlineCacheKey;
use types::indicator::IndicatorConfig;
use types::cache::CacheValue;
use types::cache::CacheItem;



pub struct CalculateIndicatorFunction;



impl CalculateIndicatorFunction {

    pub async fn calculate_indicator(
        cache_engine: Arc<Mutex<CacheEngine>>, 
        indicator_sub_key: IndicatorSubKey, 
        ignore_config: bool
    ) -> Result<Vec<Indicator>, String> {
        let offset = 1;
        let indicator_config = indicator_sub_key.indicator_config.clone();
        let kline_cache_key = KlineCacheKey::new(
            indicator_sub_key.exchange.clone(), 
            indicator_sub_key.symbol.clone(), 
            indicator_sub_key.interval.clone()
        );
        
        match indicator_config {
            IndicatorConfig::SMA(sma_config) => {
                // 如果ignore_config为true，则不使用配置的period，而是使用缓存中的所有数据
                let kline_series: Vec<Arc<CacheValue>>;
                if ignore_config {
                    kline_series = cache_engine.lock().await.get_cache_value(&kline_cache_key.into(), None, None).await;
                    // let kline_series_json = kline_series.iter().map(|kline| kline.as_kline().unwrap().close()).collect::<Vec<_>>();
                    // let kline_series_json_str = serde_json::to_string_pretty(&kline_series_json).unwrap();
                    // // println!("{}", kline_series_json_str);

                } else {
                    let period = sma_config.period as u32;
                    kline_series = cache_engine.lock().await.get_cache_value(&kline_cache_key.into(), None,Some(period + offset)).await;
                }
                let sma_list: Vec<types::indicator::sma::SMA> = CalculateIndicatorFunction::calculate_sma(sma_config, kline_series).await.unwrap();
                let sma: Vec<Indicator> = sma_list.into_iter().map(|sma| sma.into()).collect();
                // let sma_json = sma.clone().iter().map(|s| s.as_sma().unwrap().sma()).collect::<Vec<_>>();
                // let sma_json_str = serde_json::to_string_pretty(&sma_json).unwrap();
                // // println!("{}", sma_json_str);
                Ok(sma)
            }
        }
    }
}
