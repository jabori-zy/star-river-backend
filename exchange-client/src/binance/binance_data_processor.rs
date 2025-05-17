
use std::str::FromStr;
use types::market::{Kline, Exchange, KlineSeries};
use crate::binance::BinanceKlineInterval;
use strum::Display;
use strum::EnumString;
use event_center::exchange_event::{ExchangeEvent, ExchangeKlineSeriesUpdateEvent, ExchangeKlineUpdateEvent};
use utils::get_utc8_timestamp_millis;
use event_center::EventPublisher;
use utils::generate_batch_id;

#[derive(Debug, Clone, Display, EnumString, Eq, PartialEq, Hash)]
pub enum BinanceStreamEvent {
    #[strum(serialize = "kline")]
    Kline,
    #[strum(serialize = "avgPrice")]
    AvgPrice,
}



#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct KlineCacheKey {
    symbol: String,
    interval: BinanceKlineInterval,
}

#[derive(Clone, Debug)]
pub struct BinanceDataProcessor{
    // event_center: Arc<Mutex<EventCenter>>,
}

impl BinanceDataProcessor {
    pub fn new() -> Self {
        Self {
            // event_center,
        }
    }

    // 处理k线系列
    pub async fn process_kline_series(&self, symbol: &str, interval: BinanceKlineInterval, raw_data: Vec<serde_json::Value>, event_publisher: EventPublisher) {
        let klines = raw_data
            .iter()
            .map(|k| Kline {
                timestamp: k[0].as_i64().unwrap_or(0),
                open: k[1].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                high: k[2].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                low: k[3].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                close: k[4].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
                volume: k[5].as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0),
            })
            .collect::<Vec<Kline>>();
        // log::debug!("process_kline-binance: {:?}", klines);
        let kline_series = KlineSeries {
            exchange: Exchange::Binance,
            symbol: symbol.to_string(),
            interval: interval.clone().into(),
            series: klines,
        };
        // 
        let exchange_klineseries_update_event_config = ExchangeKlineSeriesUpdateEvent {
            exchange: Exchange::Binance,
            event_timestamp: get_utc8_timestamp_millis(),
            symbol: symbol.to_string(),
            interval: interval.clone().into(),
            kline_series,
            batch_id: generate_batch_id(),
        };
        // 发送k线系列更新事件
        let exchange_klineseries_update_event = ExchangeEvent::ExchangeKlineSeriesUpdate(exchange_klineseries_update_event_config).into();
        // let event_center = self.event_center.lock().await;
        // event_center.publish(exchange_klineseries_update_event).expect("发送k线系列更新事件失败");
        event_publisher.publish(exchange_klineseries_update_event);
        
        
    }


    // 处理k线数据并且更新缓存，并且发送事件
    // 发送两个事件
    // 1. k线缓存更新之后，推送整个k线数据到
    // 2. k线缓存大小事件
    pub async fn process_stream_kline(&self, raw_stream: serde_json::Value, event_publisher: EventPublisher) -> Result<(), String> {
        let k = &raw_stream["data"]["k"];
        let timestamp = k["t"].as_i64().expect("解析timestamp失败");
        let symbol = k["s"].as_str().expect("解析symbol失败");
        let open = k["o"].as_str().expect("解析open失败").parse::<f64>().expect("open不是f64");
        let high = k["h"].as_str().expect("解析high失败").parse::<f64>().expect("high不是f64");
        let low = k["l"].as_str().expect("解析low失败").parse::<f64>().expect("low不是f64");
        let close = k["c"].as_str().expect("解析close失败").parse::<f64>().expect("close不是f64");
        let volume = k["v"].as_str().expect("解析volume失败").parse::<f64>().expect("volume不是f64");
        let interval = k["i"].as_str().expect("解析interval失败").parse::<BinanceKlineInterval>().expect("interval不是KlineInterval");
        let new_kline = Kline {
            timestamp: timestamp,
            open: open,
            high: high,
            low: low,
            close: close,
            volume: volume,
        };

        let exchange_kline_update_event_config = ExchangeKlineUpdateEvent {
            exchange: Exchange::Binance,
            symbol: symbol.to_string(),
            interval: interval.clone().into(),
            kline: new_kline,
            event_timestamp: get_utc8_timestamp_millis(),
            batch_id: generate_batch_id(),
        };

        let event = ExchangeEvent::ExchangeKlineUpdate(exchange_kline_update_event_config).into();  
        // let event_center = self.event_center.lock().await;
        // event_center.publish(event).expect("发送k线更新事件失败");
        let _ = event_publisher.publish(event);

        
        Ok(())
    }


    // 处理stream流数据
    pub async fn process_stream(&self, raw_stream: serde_json::Value, event_publisher: EventPublisher) {
        if raw_stream.get("data").is_some() {
            // log::info!("process_stream-binance: {:?}", raw_stream);
            let event = raw_stream["data"]["e"].as_str().expect("解析stream_event失败");
            let stream_event = BinanceStreamEvent::from_str(event).expect("转换为BinanceStreamEvent失败");
            
            match stream_event {
                BinanceStreamEvent::Kline => {
                    // tracing::debug!("stream事件为: {:?}", stream_event);
                    self.process_stream_kline(raw_stream, event_publisher).await.expect("处理k线数据失败");
                }
                _ => {
                    tracing::warn!("不支持的事件类型: {:?}", stream_event);
                }
            }
        }
    }
    
    
}
