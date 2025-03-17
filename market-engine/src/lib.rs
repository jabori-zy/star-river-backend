
#![allow(unused_imports)]
use exchange_client::binance;
// #![allow(dead_code)]
use exchange_client::binance::{BinanceExchange, BinanceKlineInterval};
use tokio::sync::{broadcast, Mutex};
use types::market::{Exchange, KlineInterval};
use std::sync::Arc;
use exchange_client::binance::market_stream::klines;
use exchange_client::ExchangeClient;
use std::collections::HashMap;
use event_center::EventCenter;
use event_center::Event;
use types::indicator::Indicators;
use event_center::EventPublisher;
use event_center::EventReceiver;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use event_center::command_event::{CommandEvent,MarketDataEngineCommand};
use event_center::market_event::MarketEvent;
use event_center::command_event::SubscribeKlineStreamParams;
use tokio::sync::RwLock;
use event_center::command_event::{AddKlineCacheKeyParams, KlineCacheManagerCommand};
use types::cache::KlineCacheKey;
use utils::get_utc8_timestamp_millis;
use event_center::response_event::{ResponseEvent, MarketDataEngineResponse,SubscribeKlineStreamSuccessResponse};
use uuid::Uuid;

pub struct MarketDataEngineState {
    pub exchanges : HashMap<Exchange, Box<dyn ExchangeClient>>,
}

pub struct MarketDataEngine {
    pub state: Arc<RwLock<MarketDataEngineState>>,
    // 事件相关
    event_publisher: EventPublisher,
    command_event_receiver: broadcast::Receiver<Event>,
    response_event_receiver: broadcast::Receiver<Event>,

}

impl Clone for MarketDataEngine {
    fn clone(&self) -> Self {
        MarketDataEngine {
            state: self.state.clone(),
            event_publisher: self.event_publisher.clone(),
            command_event_receiver: self.command_event_receiver.resubscribe(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
        }
    }
}
impl MarketDataEngine{
    pub fn new(
        event_publisher: EventPublisher,
        command_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(MarketDataEngineState {
                exchanges: HashMap::new(),
            })),
            event_publisher: event_publisher,
            command_event_receiver: command_event_receiver,
            response_event_receiver: response_event_receiver,
        }
    }

    pub async fn run(&mut self) -> Result<(), String> {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        // 监听事件
        self.listen(internal_tx).await?;
        // 处理事件
        self.handle_events(internal_rx, self.state.clone()).await?;
        Ok(())
    }

    async fn listen(&mut self, internal_tx: mpsc::Sender<Event>) -> Result<(), String> {
        tracing::info!("市场数据引擎启动成功, 开始监听...");
        let mut response_receiver = self.response_event_receiver.resubscribe();
        let mut command_receiver = self.command_event_receiver.resubscribe();

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
        Ok(())
    }

    async fn handle_events(&mut self, mut internal_rx: mpsc::Receiver<Event>, state: Arc<RwLock<MarketDataEngineState>>) -> Result<(), String> {
        let event_publisher = self.event_publisher.clone();
        tokio::spawn(async move {
            loop {
                let event = internal_rx.recv().await.unwrap();
                match event {
                    Event::Command(command_event) => {
                        match command_event {
                            CommandEvent::MarketDataEngine(MarketDataEngineCommand::SubscribeKlineStream(params)) => {
                                Self::subscribe_kline_stream(state.clone(), params, event_publisher.clone()).await.unwrap();
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }

    fn add_cache_key(strategy_id: i32, exchange: Exchange, symbol: String, interval: KlineInterval, event_publisher: EventPublisher) {
        // 调用缓存器的订阅事件
        let cache_key = KlineCacheKey {
            exchange: exchange,
            symbol: symbol.to_string(),
            interval: interval.clone(),
        };
        let params = AddKlineCacheKeyParams {
            strategy_id,
            cache_key,
            sender: format!("strategy_{}", strategy_id),
            timestamp: get_utc8_timestamp_millis(),

        };
        let command = KlineCacheManagerCommand::AddKlineCacheKey(params);
        let command_event = CommandEvent::KlineCacheManager(command);

        event_publisher.publish(command_event.clone().into()).unwrap();
    }


    // 给策略调用的方法
    async fn subscribe_kline_stream(state: Arc<RwLock<MarketDataEngineState>>, params: SubscribeKlineStreamParams, event_publisher: EventPublisher) -> Result<(), String> {
        // tracing::debug!("市场数据引擎订阅K线流: {:?}", params);
        // 添加缓存key
        Self::add_cache_key(params.strategy_id, params.exchange.clone(), params.symbol.clone(), params.interval.clone(), event_publisher.clone());

        let exchange = params.exchange.clone();
        Self::register_exchange(&exchange, state.clone(), event_publisher.clone()).await?;
        
        let mut state = state.write().await;
        let exchange = state.exchanges.get_mut(&exchange).unwrap();

        // 先获取历史k线
        exchange.get_kline_series(&params.symbol, params.interval.clone(), Some(50), None, None).await?;
        // 再订阅k线流
        exchange.subscribe_kline_stream(&params.symbol, params.interval.clone()).await.unwrap();
        // 获取socket流
        exchange.get_socket_stream().await.unwrap();

        let request_id = params.request_id;
        tracing::warn!("市场数据引擎订阅K线流成功, 请求节点:{}, 请求id: {}", params.node_id, request_id);

        // 都成功后，发送响应事件
        let response_event = ResponseEvent::MarketDataEngine(MarketDataEngineResponse::SubscribeKlineStreamSuccess(SubscribeKlineStreamSuccessResponse {
            exchange: params.exchange,
            symbol: params.symbol,
            interval: params.interval,
            response_timestamp: get_utc8_timestamp_millis(),
            response_id: request_id,
        }));
        event_publisher.publish(response_event.clone().into()).unwrap();
        Ok(())
    }

    pub async fn get_socket_stream(&self, exchange: Exchange) -> Result<(), String> {
        let mut state = self.state.write().await;
        let exchange = state.exchanges.get_mut(&exchange).unwrap();
        exchange.get_socket_stream().await.unwrap();
        Ok(())
    }
    
    // 选择想要初始化的交易所
    pub async fn register_exchange(exchange: &Exchange, state: Arc<RwLock<MarketDataEngineState>>, event_publisher: EventPublisher) -> Result<(), String> {
        match exchange {
            Exchange::Binance => {
                // 当类型为Box<dyn Trait Bound>时，需要显式地指定类型
                let mut binance_exchange = Box::new(BinanceExchange::new(event_publisher)) as Box<dyn ExchangeClient>;
                binance_exchange.connect_websocket().await?;
                
                tracing::info!("{}交易所注册成功!", exchange);
                let mut state = state.write().await;
                state.exchanges.insert(exchange.clone(), binance_exchange);
                
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
                let state = self.state.read().await;
                let binance = state.exchanges.get(&exchange).unwrap();
                let ticker_price = binance.get_ticker_price(&symbol).await.unwrap();
                Ok(ticker_price)
            }

            _ => {
                return Err("不支持的交易所".to_string());
            }
        }
    }

    pub async fn get_kline_series(&self, exchange: Exchange, symbol: String, interval: KlineInterval, start_time: Option<u64>, end_time: Option<u64>, limit: Option<u32>) -> Result<(), String> {
        match exchange {
            Exchange::Binance => {
                let interval = BinanceKlineInterval::from(interval);
                let mut state = self.state.write().await;
                let binance = state.exchanges.get_mut(&exchange).unwrap();
                binance.get_kline_series(&symbol, interval.into(), limit, start_time, end_time).await.unwrap();
                Ok(())

            }
            _ => {
                return Err("不支持的交易所".to_string());
            }
        }

    }

    // pub async fn subscribe_kline_stream(& mut self, exchange: Exchange, symbol: String, interval: KlineInterval) -> Result<(), String> {
    //     let state = self.state.read().await;
    //     let exchange = state.exchanges.get_mut(&exchange).unwrap();
    //     let exchange = exchange.clone();
    //     // 异步执行
    //     tokio::spawn(async move {
    //             let mut exchange = exchange.lock().await;
    //             exchange.subscribe_kline_stream(&symbol, interval).await.unwrap();
    //         });
    //     Ok(())
    // }

    // 获取scoket数据
    // pub async fn get_stream(&self, exchange: Exchange) -> Result<(), String> {
    //     let state = self.state.read().await;
    //     let exchange = state.exchanges.get(&exchange).unwrap();
    //     let exchange = exchange.clone();
    //     tokio::spawn(async move {
    //         exchange.get_socket_stream().await.unwrap();
    //     });
    //     Ok(())
    // }
    
}

