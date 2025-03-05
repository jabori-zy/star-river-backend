#![allow(unused_imports)]
#![allow(dead_code)]

mod talib_bindings;
mod talib;

use std::sync::Arc;
use event_center::market_event::{MarketEvent, KlineSeriesUpdateEventInfo};
use event_center::response_event::{CacheEngineResponse, CalculateIndicatorResponse, IndicatorEngineResponse};
use tokio::sync::Mutex;
use event_center::EventCenter;
use event_center::Event;
use tokio::sync::broadcast;
use types::{cache, indicator};
use types::indicator::SMASeries;
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
use event_center::command_event::{CommandEvent, GetSubscribedIndicatorParams, IndicatorCacheManagerCommand, IndicatorEngineCommand, CalculateIndicatorParams};
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
    event_publisher: EventPublisher,
    kline_series_cache: Arc<RwLock<HashMap<Uuid, KlineSeriesUpdateEventInfo>>>,
}

impl IndicatorEngine {

    pub fn new(
        event_publisher: EventPublisher,
    ) -> Self {
        let talib = Arc::new(TALib::init().unwrap());
        Self { 
            talib,
            event_publisher,
            kline_series_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(
        &self,
        command_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        self.listen(command_event_receiver, response_event_receiver, internal_tx).await;

        let mut indicator_engine = self.clone();
        tokio::spawn(async move {
            indicator_engine.handle_events(internal_rx).await;
        });
    }

    async fn listen(
        &self, 
        mut command_receiver: broadcast::Receiver<Event>,
        mut response_receiver: broadcast::Receiver<Event>,
        internal_tx: mpsc::Sender<Event>,
    ) {
        tracing::info!("指标引擎启动成功, 开始监听...");

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = response_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                    Ok(event) = command_receiver.recv() => {
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
                // Event::Response(response_event) => {
                //     self.handle_response_event(response_event).await;
                // }
                Event::Command(command_event) => {
                    self.handle_command_event(command_event).await;
                }
                _ => {}
            }
        }
    }

    // 处理命令事件
    async fn handle_command_event(&mut self, command_event: CommandEvent) {
        match command_event {
            CommandEvent::IndicatorEngine(indicator_engine_command) => {
                match indicator_engine_command {
                    IndicatorEngineCommand::CalculateIndicator(calculate_indicator_params) => {
                        self.calculate_indicator(calculate_indicator_params).await;
                    }
                }
            }
            _ => {}
        }
    }

    // async fn handle_response_event(&mut self, response_event: ResponseEvent) {
    //     match response_event {
    //         ResponseEvent::CacheEngine(cache_engine_response) => {
    //             match cache_engine_response {
    //                 // 获取到返回的指标列表，开始计算
    //                 CacheEngineResponse::SubscribedIndicator(subscribed_indicator_response) => {
    //                     let response_id = subscribed_indicator_response.response_id;
    //                     let indicator_cache_key_list = subscribed_indicator_response.indicator_cache_key_list;
    //                     self.calculate_indicator(response_id, indicator_cache_key_list).await;
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    // }

    async fn calculate_indicator(&mut self, calculate_indicator_params: CalculateIndicatorParams) {
        tracing::info!("接收到计算指标命令: {:?}", calculate_indicator_params);
        let indicator = calculate_indicator_params.indicator.clone();

        match indicator {
            Indicators::SimpleMovingAverage(sma_config) => {
                let period = sma_config.period;
                IndicatorEngine::calculate_sma(self.talib.clone(), &period, calculate_indicator_params, self.event_publisher.clone()).await.unwrap();
            }
            _ => {}
        }


        
    }

    async fn calculate_sma(talib: Arc<TALib>, period: &i32, calculate_params: CalculateIndicatorParams, event_publisher: EventPublisher) -> Result<(), String> {
        let kline_series = calculate_params.kline_series.clone();
        let timestamp_list: Vec<i64> = kline_series.series.iter().map(|v| v.timestamp).collect();  
        let close: Vec<f64> = kline_series.series.iter().map(|v| v.close).collect();

        let sma = talib.sma(&close, *period)?;
        // log::info!("{}: sma: {:?}", event.symbol,sma);
        // 将timestamp_list和sma组合成SMA结构体
        let sma_list: Vec<SMA> = timestamp_list.iter().zip(sma.iter()).map(|(timestamp, sma)| SMA { timestamp: *timestamp, value: *sma }).collect();
        // log::info!("{}: sma_list: {:?}", kline_event.symbol,sma_list);
        
        let sma_series = SMASeries {
            exchange: calculate_params.exchange.clone(),
            symbol: calculate_params.symbol.clone(),
            kline_interval: calculate_params.interval.clone(),
            sma_config: SMAConfig { period: *period },
            data: sma_list,
        };

        let response = CalculateIndicatorResponse {
            exchange: calculate_params.exchange.clone(),
            symbol: calculate_params.symbol.clone(),
            interval: calculate_params.interval.clone(),
            indicator: calculate_params.indicator.clone(),
            value: Box::new(sma_series),
            response_timestamp: get_utc8_timestamp(),
            response_id: calculate_params.request_id.clone(),
            batch_id: calculate_params.batch_id.clone(),
        };
        
        let response_event = ResponseEvent::IndicatorEngine(IndicatorEngineResponse::CalculateIndicatorFinish(response));
        let result = event_publisher.publish(response_event.clone().into());
        // tracing::debug!("响应事件: {:?}", response_event);
        if result.is_err() {
            tracing::error!("响应事件失败: {:?}", result.err().unwrap());
        }

        Ok(())
    }

    fn payload_to_list(payload: &serde_json::Value) -> Vec<f64> {
        let list = payload["list"].as_array().unwrap();
        list.iter().map(|v| v.as_f64().unwrap()).collect()
    }
}



