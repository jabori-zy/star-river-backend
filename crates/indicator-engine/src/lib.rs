#![allow(unused_imports)]
#![allow(dead_code)]

mod talib_bindings;
mod talib;

use std::sync::Arc;
use event_center::market_event::{MarketEvent, KlineSeriesEventInfo};
use event_center::response_event::{self, CacheEngineResponse};
use tokio::sync::Mutex;
use event_center::EventCenter;
use event_center::Event;
use tokio::sync::broadcast;
use types::{cache, indicator};
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
use event_center::response_event::ResponseEvent;
use strum::{Display, EnumString};
use types::indicator::Indicators;
use crate::talib::TALib;
use data_cache::CacheEngine;
use event_center::command_event::{CommandEvent, GetSubscribedIndicatorParams, IndicatorCacheManagerCommand};
use utils::get_utc8_timestamp;
use event_center::EventPublisher;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use types::cache::IndicatorCacheKey;
use std::sync::RwLock;
use futures::future::join_all;

#[derive(Clone)]
pub struct IndicatorEngine{
    // 需要计算的指标
    talib: Arc<TALib>,
    cache_engine: Arc<Mutex<CacheEngine>>,
    event_publisher: EventPublisher,
    kline_series_cache: Arc<RwLock<HashMap<Uuid, KlineSeriesEventInfo>>>,
}

impl IndicatorEngine {

    pub fn new(
        cache_engine: Arc<Mutex<CacheEngine>>, 
        event_publisher: EventPublisher,
    ) -> Self {
        let talib = Arc::new(TALib::init().unwrap());
        Self { 
            talib,
            cache_engine,
            event_publisher,
            kline_series_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(
        &self,
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        self.listen(market_event_receiver, response_event_receiver, internal_tx).await;

        let mut indicator_engine = self.clone();
        tokio::spawn(async move {
            indicator_engine.handle_events(internal_rx).await;
        });
    }

    async fn listen(
        &self, 
        mut market_receiver: broadcast::Receiver<Event>,
        mut response_receiver: broadcast::Receiver<Event>,
        internal_tx: mpsc::Sender<Event>,
    ) {
        tracing::info!("指标引擎启动成功, 开始监听...");

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = market_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                    Ok(event) = response_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });

    }

    // 处理接收到的事件
    async fn handle_events(&mut self, mut internal_rx: mpsc::Receiver<Event>) {
        loop {
            let event = internal_rx.recv().await.unwrap();
            match event {
                Event::Market(market_event) => {
                    self.handle_market_event(market_event).await;
                }
                Event::Response(response_event) => {
                    self.handle_response_event(response_event).await;
                }
                _ => {}
            }
        }
    }

    async fn handle_response_event(&mut self, response_event: ResponseEvent) {
        match response_event {
            ResponseEvent::CacheEngine(cache_engine_response) => {
                match cache_engine_response {
                    // 获取到返回的指标列表，开始计算
                    CacheEngineResponse::SubscribedIndicator(subscribed_indicator_response) => {
                        let response_id = subscribed_indicator_response.response_id;
                        let indicator_cache_key_list = subscribed_indicator_response.indicator_cache_key_list;
                        self.calculate_indicator(response_id, indicator_cache_key_list).await;
                    }
                }
            }
        }
    }

    async fn handle_market_event(&mut self, market_event: MarketEvent) {
        match market_event {
            MarketEvent::KlineSeriesUpdate(kline_series_event) => {
                self.get_subscribed_indicator(kline_series_event).await;
            }
            _ => {}
        }
    }

    async fn calculate_indicator(&mut self, response_id: Uuid, indicator_cache_key_list: Vec<IndicatorCacheKey>) {

        if indicator_cache_key_list.is_empty() {
            // 如果指标列表为空，则跳过计算，并删除该条缓存
            let mut kline_series_cache = self.kline_series_cache.write().expect("获取kline_series_cache写锁失败");
            kline_series_cache.remove(&response_id);
            return;
        }

        //获取k线系列事件
        let kline_series_event = {
            let kline_series_cache = self.kline_series_cache.read().expect("获取kline_series_cache读锁失败");
            kline_series_cache.get(&response_id).expect("request_id不存在").clone()
        };

        for indicator_cache_key in indicator_cache_key_list {
            let kline_series_event = kline_series_event.clone();
            let event_publisher = self.event_publisher.clone();
            let talib = self.talib.clone();
            
            tokio::spawn(async move {
                IndicatorEngine::handle_indicator(talib, indicator_cache_key, kline_series_event, event_publisher).await;
            });
        }

        // 计算完成后，删除缓存
        let mut kline_series_cache = self.kline_series_cache.write().expect("获取kline_series_cache写锁失败");
        kline_series_cache.remove(&response_id);
        
        

        
    }

    async fn handle_indicator(talib: Arc<TALib>, indicator_cache_key: IndicatorCacheKey, kline_series_event: KlineSeriesEventInfo, event_publisher: EventPublisher) {
        match indicator_cache_key.indicator {
            Indicators::SimpleMovingAverage(sma_config) => {
                let sma = IndicatorEngine::calculate_sma(talib, &sma_config.period, kline_series_event, event_publisher).await;
                // tracing::info!("sma: {:?}", sma);
            }
            _ => {}
        }
    }

    async fn get_subscribed_indicator(&mut self, kline_series_event: KlineSeriesEventInfo) {
        // 生成请求id
        tracing::debug!("接收K线系列更新事件, 生成请求id, 发送命令获取k线订阅的指标列表");
        let request_id = Uuid::new_v4();
        let params = GetSubscribedIndicatorParams {
            exchange: kline_series_event.clone().exchange,
            symbol: kline_series_event.clone().symbol,
            interval: kline_series_event.clone().interval,
            sender: "indicator_engine".to_string(),
            timestamp: get_utc8_timestamp(),
            request_id: request_id.clone()
        };

        let mut kline_series_cache = self.kline_series_cache.write().expect("获取kline_series_cache写锁失败");
        kline_series_cache.insert(request_id, kline_series_event.clone());

        let command = CommandEvent::IndicatorCacheManager(IndicatorCacheManagerCommand::GetSubscribedIndicator(params));
        let event_publisher = self.event_publisher.clone();

        // 向缓存引擎发送命令：获取k线订阅的指标列表
        event_publisher.publish(command.into()).unwrap();





    }


    async fn calculate_sma(talib: Arc<TALib>, period: &i32, kline_event: KlineSeriesEventInfo, event_publisher: EventPublisher) -> Result<(), String> {
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
        // self.event_center.lock().await.publish(sma_event).expect("发布指标事件失败");
        let result = event_publisher.publish(sma_event.into());
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



