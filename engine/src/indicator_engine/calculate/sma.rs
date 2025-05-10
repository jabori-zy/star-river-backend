use super::CalculateIndicatorFunction;
use types::indicator::sma::{SMA, SMAConfig};
use types::cache::CacheValue;
use std::sync::Arc;
use crate::indicator_engine::talib::TALib;


impl CalculateIndicatorFunction {

    pub async fn calculate_sma(sma_config: SMAConfig, kline_series: Vec<Arc<CacheValue>>) -> Result<Vec<SMA>, String> {
        let timestamp_list: Vec<i64> = kline_series.iter().map(|v| v.as_kline().unwrap().timestamp).collect(); 
        let close: Vec<f64> = kline_series.iter().map(|v| v.as_kline().unwrap().close).collect();

        let sma = TALib::sma(&close, sma_config.period)?;
        // log::info!("{}: sma: {:?}", event.symbol,sma);
        // 将timestamp_list和sma组合成SMA结构体
        let sma_list: Vec<SMA> = timestamp_list.iter().zip(sma.iter()).map(|(timestamp, sma)| SMA { timestamp: *timestamp, sma: *sma }).collect();
        Ok(sma_list)
    }
}
