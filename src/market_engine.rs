
#![allow(unused_imports)]
// #![allow(dead_code)]
use exchange_client::binance::{BinanceExchange, BinanceKlineInterval};
use tokio::sync::Mutex;
use types::market::{Exchange, KlineInterval};
use std::sync::Arc;
use exchange_client::binance::market_stream::klines;
use exchange_client::ExchangeClient;
use std::collections::HashMap;
use event_center::EventCenter;

#[derive(Clone)]
pub struct MarketDataEngine {
    pub exchanges : HashMap<Exchange, Arc<Mutex<Box<dyn ExchangeClient>>>>,
}

impl MarketDataEngine{
    pub fn new() -> Self {
        Self {
            exchanges: HashMap::new(),
        }
    }

    pub async fn get_binance_exchange(&self) -> Result<Arc<Mutex<BinanceExchange>>, String> {
        let exchange = self.exchanges.get(&Exchange::Binance)
            .ok_or("Binance exchange not found".to_string())?;
        
        let exchange = exchange.lock().await;
        if let Some(binance) = exchange.as_any().downcast_ref::<BinanceExchange>() {
            Ok(Arc::new(Mutex::new(binance.clone())))
        } else {
            Err("Failed to downcast to BinanceExchange".to_string())
        }
    }
    
    // 选择想要初始化的交易所
    pub async fn register_exchange(&mut self, exchange: Exchange, event_center: Arc<Mutex<EventCenter>>) -> Result<(), String> {
        match exchange {
            Exchange::Binance => {
                // 当类型为Box<dyn Trait Bound>时，需要显式地指定类型
                let mut binance_exchange = Box::new(BinanceExchange::new(event_center)) as Box<dyn ExchangeClient>;
                binance_exchange.connect_websocket().await?;
                
                tracing::info!("{}交易所注册成功!", exchange);
                self.exchanges.insert(exchange, Arc::new(Mutex::new(binance_exchange)));
                
                Ok(())

            }

            _ => {
                return Err("不支持的交易所".to_string());
            }
       }
    }


    pub async fn get_ticker_price(&self, exchange: Exchange, symbol: String) -> Result<serde_json::Value, String> {
        match exchange {
            Exchange::Binance => {
                let binance = self.exchanges.get(&exchange).unwrap();
                let binance = binance.lock().await;
                let ticker_price = binance.get_ticker_price(&symbol).await.unwrap();
                Ok(ticker_price)
            }

            _ => {
                return Err("不支持的交易所".to_string());
            }
        }
    }

    pub async fn get_kline_buffer(&self, exchange: Exchange, symbol: String, interval: KlineInterval, start_time: Option<u64>, end_time: Option<u64>, limit: Option<u32>) -> Result<(), String> {
        match exchange {
            Exchange::Binance => {
                let interval = BinanceKlineInterval::from(interval);
                let binance = self.exchanges.get(&exchange).unwrap();
                let mut binance = binance.lock().await;
                binance.get_klines(&symbol, interval.into(), limit, start_time, end_time).await.unwrap();
                Ok(())

            }
            _ => {
                return Err("不支持的交易所".to_string());
            }
        }

    }

    pub async fn subscribe_kline_stream(& mut self, exchange: Exchange, symbol: String, interval: KlineInterval) -> Result<(), String> {
        if let Some(exchange) = self.exchanges.get_mut(&exchange){
            let exchange = exchange.clone();
            // 异步执行
            tokio::spawn(async move {
                let mut exchange = exchange.lock().await;
                exchange.subscribe_kline_stream(&symbol, interval).await.unwrap();
            });
            Ok(())
        } else {
            Err("交易所不存在".to_string())
        }
    }

    // 获取scoket数据
    pub async fn get_stream(&self, exchange: Exchange) -> Result<(), String> {
        if let Some(exchange) = self.exchanges.get(&exchange){
            let exchange = exchange.clone();
            tokio::spawn(async move {
                let mut exchange = exchange.lock().await;
                exchange.get_socket_stream().await.unwrap();
            });
            Ok(())
        } else {
            Err("交易所不存在".to_string())
        }
    }
}

