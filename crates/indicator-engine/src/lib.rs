#![allow(unused_imports)]
#![allow(dead_code)]

mod talib_bindings;
mod talib;

use std::sync::Arc;
use event_center::market_event::{MarketEvent, KlineSeriesEventInfo};
use tokio::sync::Mutex;
use event_center::EventCenter;
use event_center::Event;
use tokio::sync::broadcast;
use types::cache;
use types::indicator::SMABuffer;
use types::market::{Exchange, KlineInterval};
use std::collections::HashMap;
use uuid::Uuid;
use std::str::FromStr;
use utils::timestamp_to_utc8;
use event_center::Channel;
use types::indicator_config::SMAConfig;
use types::indicator::SMA;
use event_center::indicator_event::IndicatorEvent;
use strum::{Display, EnumString};
use types::indicator::Indicators;
use crate::talib::TALib;
use data_cache::CacheEngine;



pub struct IndicatorEngine{
    // 需要计算的指标
    talib: Arc<TALib>,
    event_center: Arc<Mutex<EventCenter>>,
    cache_engine: Arc<Mutex<CacheEngine>>,
    event_publisher: broadcast::Sender<Event>,
    event_receiver: broadcast::Receiver<Event>

}

impl IndicatorEngine {

    pub fn new(
        event_center: Arc<Mutex<EventCenter>>, 
        cache_engine: Arc<Mutex<CacheEngine>>, 
        event_publisher: broadcast::Sender<Event>,
        event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let talib = Arc::new(TALib::init().unwrap());
        Self { 
            talib, 
            event_center,
            cache_engine,
            event_publisher,
            event_receiver
        }
    }

    pub async fn listen(&self) {
        let mut receiver = {
            let event_center = self.event_center.lock().await;
            event_center.subscribe(Channel::Market).unwrap()
        };

        while let Ok(event) = receiver.recv().await {
            match event {
                Event::Market(MarketEvent::KlineSeriesUpdate(kline_series_event)) => {
                    self.calculate_indicator(kline_series_event).await;
                }
                _ => {}
            }
        }

    }

    async fn calculate_indicator(&self, kline_series_event: KlineSeriesEventInfo) {

        let cache_engine = self.cache_engine.lock().await;
        
        let indicator_cache_manager = cache_engine.indicator_cache_manager.lock().await;
        
        let indicator_list = indicator_cache_manager.get_kline_series_sub_indicator(
            kline_series_event.clone().exchange,
            kline_series_event.clone().symbol,
            kline_series_event.clone().interval
        );
        tracing::debug!("获取k线订阅的指标列表: {:?}", indicator_list);

        

        for indicator_cache_key in indicator_list {
            match indicator_cache_key.indicator {
                Indicators::SimpleMovingAverage(config) => {
                    let talib = self.talib.clone();
                    let indicator_publisher = self.event_publisher.clone();
                    IndicatorEngine::calculate_sma(talib, &config.period, kline_series_event.clone(), indicator_publisher).await.unwrap();
                }
            }

        }
        
        




    }


    async fn calculate_sma(talib: Arc<TALib>, period: &i32, kline_event: KlineSeriesEventInfo, indicator_publisher: broadcast::Sender<Event>) -> Result<(), String> {
        let kline_series = kline_event.kline_series.clone();
        let timestamp_list: Vec<i64> = kline_series.series.iter().map(|v| v.timestamp).collect();  
        let close: Vec<f64> = kline_series.series.iter().map(|v| v.close).collect();
        // log::info!("timestamp_list: {:?}", timestamp_list);
        // log::info!("{}: close: {:?}", event.symbol,close);
        let sma = talib.sma(&close, *period)?;
        // log::info!("{}: sma: {:?}", event.symbol,sma);
        // 将timestamp_list和sma组合成SMA结构体
        let sma_list: Vec<SMA> = timestamp_list.iter().zip(sma.iter()).map(|(timestamp, sma)| SMA { timestamp: *timestamp, value: *sma }).collect();
        // log::info!("{}: sma_list: {:?}", kline_event.symbol,sma_list);
        
        let sma_buffer = SMABuffer {
            exchange: kline_event.exchange.clone(),
            symbol: kline_event.symbol.clone(),
            kline_interval: kline_event.interval.clone(),
            sma_config: SMAConfig { period: *period },
            buffer: sma_list,
        };

        let sma_event = IndicatorEvent::SMAUpdate(sma_buffer);
        tracing::info!("发布指标事件: {:?}", sma_event);
        // self.event_center.lock().await.publish(sma_event).expect("发布指标事件失败");
        let result = indicator_publisher.send(sma_event.into());
        if result.is_err() {
            tracing::error!("发布指标事件失败: {:?}", result.err().unwrap());
        }

        Ok(())
    }

    fn payload_to_list(payload: &serde_json::Value) -> Vec<f64> {
        let list = payload["list"].as_array().unwrap();
        list.iter().map(|v| v.as_f64().unwrap()).collect()
    }
}



