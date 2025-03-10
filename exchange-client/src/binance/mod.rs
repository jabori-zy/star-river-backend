pub mod websocket;
pub(crate) mod url;
pub mod binance_http_client;
pub mod binance_ws_client;
pub mod market_stream;
pub mod binance_data_processor;
use std::any::Any;
use std::sync::atomic::AtomicBool;

use binance_http_client::BinanceHttpClient;
use binance_ws_client::BinanceWsClient;
use strum::Display;
use serde::{Deserialize, Serialize};
use crate::utils::deserialize_string_to_f64;
use types::market::{Kline, TickerPrice, Exchange, KlineInterval};
use strum::EnumString;
use std::sync::Arc;
use tokio::sync::Mutex;
use binance_ws_client::WebSocketState;
use crate::ExchangeClient;
use async_trait::async_trait;
use futures::StreamExt;
use crate::binance::market_stream::klines;
use crate::binance::binance_data_processor::BinanceDataProcessor;
use event_center::command_event::{CommandEvent, KlineCacheManagerCommand, SubscribeKlineParams, IndicatorCacheManagerCommand, SubscribeIndicatorParams};
use types::cache::{KlineCacheKey, IndicatorCacheKey};
use utils::get_utc8_timestamp_millis;
use types::indicator::Indicators;
use event_center::EventPublisher;


#[derive(Clone, Display, Serialize, Deserialize, Debug, EnumString, Eq, PartialEq, Hash)]
pub enum BinanceKlineInterval {
    #[strum(serialize = "1m")]
    Minutes1,
    #[strum(serialize = "5m")]
    Minutes5,
    #[strum(serialize = "15m")]
    Minutes15,
    #[strum(serialize = "30m")]
    Minutes30,
    #[strum(serialize = "1h")]
    Hours1,
    #[strum(serialize = "2h")]
    Hours2,
    #[strum(serialize = "4h")]
    Hours4,
    #[strum(serialize = "6h")]
    Hours6,
    #[strum(serialize = "8h")]
    Hours8,
    #[strum(serialize = "12h")]
    Hours12,
    #[strum(serialize = "1d")]
    Days1,
    #[strum(serialize = "1w")]
    Weeks1,
    #[strum(serialize = "1M")]
    Months1,
}

// 将KlineInterval转换为BinanceKlineInterval
impl From<KlineInterval> for BinanceKlineInterval {
    fn from(interval: KlineInterval) -> Self {
        match interval {
            KlineInterval::Minutes1 => BinanceKlineInterval::Minutes1,
            KlineInterval::Minutes5 => BinanceKlineInterval::Minutes5,
            KlineInterval::Minutes15 => BinanceKlineInterval::Minutes15,
            KlineInterval::Minutes30 => BinanceKlineInterval::Minutes30,
            KlineInterval::Hours1 => BinanceKlineInterval::Hours1,
            KlineInterval::Hours2 => BinanceKlineInterval::Hours2,
            KlineInterval::Hours4 => BinanceKlineInterval::Hours4,
            KlineInterval::Hours6 => BinanceKlineInterval::Hours6,
            KlineInterval::Hours8 => BinanceKlineInterval::Hours8,
            KlineInterval::Hours12 => BinanceKlineInterval::Hours12,
            KlineInterval::Days1 => BinanceKlineInterval::Days1,
            KlineInterval::Weeks1 => BinanceKlineInterval::Weeks1,
            KlineInterval::Months1 => BinanceKlineInterval::Months1,

        }
    }
}

// 将BinanceKlineInterval转换为KlineInterval
impl Into<KlineInterval> for BinanceKlineInterval {
    fn into(self) -> KlineInterval {
        match self {
            BinanceKlineInterval::Minutes1 => KlineInterval::Minutes1,
            BinanceKlineInterval::Minutes5 => KlineInterval::Minutes5,
            BinanceKlineInterval::Minutes15 => KlineInterval::Minutes15,
            BinanceKlineInterval::Minutes30 => KlineInterval::Minutes30,
            BinanceKlineInterval::Hours1 => KlineInterval::Hours1,
            BinanceKlineInterval::Hours2 => KlineInterval::Hours2,
            BinanceKlineInterval::Hours4 => KlineInterval::Hours4,
            BinanceKlineInterval::Hours6 => KlineInterval::Hours6,
            BinanceKlineInterval::Hours8 => KlineInterval::Hours8,
            BinanceKlineInterval::Hours12 => KlineInterval::Hours12,
            BinanceKlineInterval::Days1 => KlineInterval::Days1,
            BinanceKlineInterval::Weeks1 => KlineInterval::Weeks1,
            BinanceKlineInterval::Months1 => KlineInterval::Months1,
        }
    }
}





#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BinanceTickerPrice {
    pub symbol: String,
    #[serde(deserialize_with = "deserialize_string_to_f64")]
    pub price: f64,
    #[serde(skip_deserializing)]
    pub timestamp: i64,
}


impl From<BinanceTickerPrice> for TickerPrice {
    fn from(ticker_price: BinanceTickerPrice) -> Self {
        Self {
            exchange: Exchange::Binance,
            symbol: ticker_price.symbol,
            price: ticker_price.price,
            timestamp: ticker_price.timestamp,
        }

    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceKline {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl From<BinanceKline> for Kline {
    fn from(kline: BinanceKline) -> Self {
        Self {
            timestamp: kline.timestamp,
            open: kline.open,
            high: kline.high,
            low: kline.low,
            close: kline.close,
            volume: kline.volume,
        }
    }

}



// 交易所信息
#[derive(Clone, Eq, PartialEq)]
pub struct BinanceExchangeInfo {
    pub timezone: String,
    pub server_time: i64,
    pub symbols: Option<Vec<String>>,
}

// 交易所
#[derive(Clone)]
pub struct BinanceExchange {
    pub server_time: Option<i64>,
    pub info: Option<BinanceExchangeInfo>,
    http_client: BinanceHttpClient,
    websocket_state: Arc<Mutex<Option<WebSocketState>>>, // 可以在线程间传递
    data_processor: Arc<Mutex<BinanceDataProcessor>>,
    is_process_stream: Arc<AtomicBool>,
    event_publisher: EventPublisher,
}

#[async_trait]
impl ExchangeClient for BinanceExchange {
    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String> {
        let ticker_price = self.http_client.get_ticker_price(symbol).await?;
        Ok(ticker_price)
    }

    // 获取k线系列
    async fn get_kline_series(&mut self, symbol: &str, interval: KlineInterval, limit: Option<u32>, start_time: Option<u64>, end_time: Option<u64>) -> Result<(), String> {
        // 调用缓存器的订阅事件
        let cache_key = KlineCacheKey {
            exchange: Exchange::Binance,
            symbol: symbol.to_string(),
            interval: interval.clone(),
        };
        let params = SubscribeKlineParams {
            cache_key,
            sender: "binance_exchange".to_string(),
            timestamp: get_utc8_timestamp_millis(),

        };
        let command = KlineCacheManagerCommand::SubscribeKline(params);
        let command_event = CommandEvent::KlineCacheManager(command);

        // 使用binance_publisher发送命令
        // let _ = self.binance_publisher.send(command_event.clone().into());
        let _ = self.event_publisher.publish(command_event.clone().into());

        let binance_interval = BinanceKlineInterval::from(interval);

        let klines = self.http_client.get_kline(symbol, binance_interval.clone(), limit, start_time, end_time).await?;
        // 发送到数据处理器，处理数据
        let data_processor = self.data_processor.lock().await;
        data_processor.process_kline_series(symbol, binance_interval, klines, self.event_publisher.clone()).await;
        Ok(())

    }

    async fn connect_websocket(&mut self) -> Result<(), String> {
        let (websocket_state, _) = BinanceWsClient::connect_default().await.unwrap();
        self.websocket_state = Arc::new(Mutex::new(Some(websocket_state)));
        Ok(())
    }

    // 订阅k线流
    async fn subscribe_kline_stream(&mut self, symbol: &str, interval: KlineInterval) -> Result<(), String> {    
        tracing::debug!("订阅k线流, symbol: {:?}, interval: {:?}", symbol, interval);
        let binance_interval = BinanceKlineInterval::from(interval.clone());

        let mut websocket_state = self.websocket_state.lock().await;
        if let Some(state) = websocket_state.as_mut() {
            tracing::debug!("订阅k线流, symbol: {:?}, interval: {:?}", symbol, interval);
            state.subscribe([&klines(symbol, binance_interval).into()]).await;
        }
        Ok(())
    }

    // 获取socket流，并处理数据
    async fn get_socket_stream(&mut self) -> Result<(), String> {
        // 判断当前是否正在处理流
        if self.is_process_stream.load(std::sync::atomic::Ordering::Relaxed) {
            tracing::warn!("binance已开始处理流数据, 无需重复获取!");
            return Ok(());
        }
        tracing::debug!("开始binance处理流数据");
        // 如果当前没有处理流，则开始处理流,设置状态为true
        self.is_process_stream.store(true, std::sync::atomic::Ordering::Relaxed);

        let websocket_state = self.websocket_state.clone();
        let data_processor = self.data_processor.clone();


        let binance_publisher = self.event_publisher.clone();
        let future = async move {
            loop {
                let receive_message = {
                    let mut websocket_state = websocket_state.lock().await;
                    if let Some(state) = websocket_state.as_mut() {
                        state.as_mut().next().await
                    } else {
                        None
                    }
                };  // 锁在这里被释放
                
                // 处理原始数据
                if let Some(Ok(msg)) = receive_message {
                    let stream_json = serde_json::from_str::<serde_json::Value>(&msg.to_string()).unwrap();
                    // log::debug!("收到stream数据: {:?}", stream_json);
                    let data_processor = data_processor.lock().await;
                    data_processor.process_stream(stream_json, binance_publisher.clone()).await;  
                }
            }
        };
        tokio::spawn(future);
        Ok(())
    }
}


impl BinanceExchange {
    pub fn new(event_publisher: EventPublisher) -> Self {
        Self {
            server_time: None,
            info: None,
            http_client: BinanceHttpClient::new(),
            websocket_state: Arc::new(Mutex::new(None)),
            data_processor: Arc::new(Mutex::new(BinanceDataProcessor::new())),
            is_process_stream: Arc::new(AtomicBool::new(false)),
            event_publisher,
        }
    }
    
    
    pub async fn init_exchange(&mut self) -> Result<(), String> {
        tracing::debug!("正在初始化binance交易所...");
        let (websocket_state, _) = BinanceWsClient::connect_default().await.unwrap();
        self.websocket_state = Arc::new(Mutex::new(Some(websocket_state)));
        tracing::debug!("Binance 初始化成功！");
        

        Ok(())
        
    }

    pub async fn subscribe_indicator(&self, symbol: &str, interval: KlineInterval, indicator: Indicators) -> Result<(), String> {
        let ind_cache_key = IndicatorCacheKey {
            exchange: Exchange::Binance,
            symbol: symbol.to_string(),
            interval,
            indicator,
            
        };
        let params = SubscribeIndicatorParams {
            cache_key: ind_cache_key,
            sender: "binance_exchange".to_string(),
            timestamp: get_utc8_timestamp_millis(),

        };
        let command = IndicatorCacheManagerCommand::SubscribeIndicator(params);
        let command_event = CommandEvent::IndicatorCacheManager(command);
        let _ = self.event_publisher.publish(command_event.into());

        Ok(())
    }

        

}



