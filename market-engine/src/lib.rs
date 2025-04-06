
// #![allow(unused_imports)]
// #![allow(dead_code)]
use exchange_client::binance::{BinanceExchange, BinanceKlineInterval};
use tokio::sync::{broadcast, Mutex};
use types::market::{Exchange, KlineInterval};
use std::sync::Arc;
use exchange_client::ExchangeClient;
use std::collections::HashMap;
use event_center::Event;
use event_center::EventPublisher;
use tokio::sync::mpsc;
use event_center::command_event::{CommandEvent,MarketDataEngineCommand};
use event_center::command_event::{SubscribeKlineStreamParams, UnsubscribeKlineStreamParams};
use tokio::sync::RwLock;
use event_center::command_event::{AddKlineCacheKeyParams, KlineCacheManagerCommand};
use types::cache::KlineCacheKey;
use utils::get_utc8_timestamp_millis;
use event_center::response_event::{ResponseEvent, MarketDataEngineResponse,SubscribeKlineStreamSuccessResponse, UnsubscribeKlineStreamSuccessResponse};
use exchange_client::metatrader5::MetaTrader5;
use exchange_client::ExchangeManager;

pub struct MarketDataEngineState {
    pub exchanges : HashMap<Exchange, Box<dyn ExchangeClient>>,
}

pub struct MarketDataEngine {
    pub state: Arc<RwLock<MarketDataEngineState>>,

    // 事件相关
    event_publisher: EventPublisher,
    command_event_receiver: broadcast::Receiver<Event>,
    response_event_receiver: broadcast::Receiver<Event>,
    exchange_manager: Arc<Mutex<ExchangeManager>>,

}

impl Clone for MarketDataEngine {
    fn clone(&self) -> Self {
        MarketDataEngine {
            state: self.state.clone(),
            event_publisher: self.event_publisher.clone(),
            command_event_receiver: self.command_event_receiver.resubscribe(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
            exchange_manager: self.exchange_manager.clone(),
        }
    }
}
impl MarketDataEngine{
    pub fn new(
        event_publisher: EventPublisher,
        command_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        exchange_manager: Arc<Mutex<ExchangeManager>>,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(MarketDataEngineState {
                exchanges: HashMap::new(),
            })),
            event_publisher: event_publisher,
            command_event_receiver: command_event_receiver,
            response_event_receiver: response_event_receiver,
            exchange_manager: exchange_manager,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        // 监听事件
        self.listen(internal_tx).await?;
        // 处理事件
        self.handle_events(internal_rx, self.state.clone(), self.exchange_manager.clone()).await?;
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

    async fn handle_events(&mut self, mut internal_rx: mpsc::Receiver<Event>, state: Arc<RwLock<MarketDataEngineState>>, exchange_manager: Arc<Mutex<ExchangeManager>>) -> Result<(), String> {
        let event_publisher = self.event_publisher.clone();
        tokio::spawn(async move {
            loop {
                let event = internal_rx.recv().await.unwrap();
                match event {
                    Event::Command(command_event) => {
                        match command_event {
                            CommandEvent::MarketDataEngine(MarketDataEngineCommand::SubscribeKlineStream(params)) => {
                                Self::subscribe_kline_stream(state.clone(), params, event_publisher.clone(), exchange_manager.clone()).await.unwrap();
                            }
                            CommandEvent::MarketDataEngine(MarketDataEngineCommand::UnsubscribeKlineStream(params)) => {
                                Self::unsubscribe_kline_stream(state.clone(), params, event_publisher.clone(), exchange_manager.clone()).await.unwrap();
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
    async fn subscribe_kline_stream(state: Arc<RwLock<MarketDataEngineState>>, params: SubscribeKlineStreamParams, event_publisher: EventPublisher, exchange_manager: Arc<Mutex<ExchangeManager>>) -> Result<(), String> {
        // tracing::debug!("市场数据引擎订阅K线流: {:?}", params);
        // 添加缓存key
        Self::add_cache_key(params.strategy_id, params.exchange.clone(), params.symbol.clone(), params.interval.clone(), event_publisher.clone());

        // let exchange = params.exchange.clone();
        // Self::register_exchange(&exchange, state.clone(), event_publisher.clone()).await?;
        
        // let mut state = state.write().await;
        // let exchange = state.exchanges.get_mut(&exchange).unwrap();
        let exchange_manager_guard = exchange_manager.lock().await;
        // 检查是否已经注册
        // 创建无限循环，只有当已注册时才退出
        loop {
            if exchange_manager_guard.is_registered(&params.exchange).await {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        let exchange = exchange_manager_guard.get_exchange_ref(&params.exchange).await?;

        // 先获取历史k线
        // k线长度设置
        exchange.get_kline_series(&params.symbol, params.interval.clone(), Some(20)).await?;
        // 再订阅k线流
        exchange.subscribe_kline_stream(&params.symbol, params.interval.clone(), params.frequency).await.unwrap();
        // 获取socket流
        exchange.get_socket_stream().await.unwrap();

        let request_id = params.request_id;
        tracing::debug!("市场数据引擎订阅K线流成功, 请求节点:{}, 请求id: {}", params.node_id, request_id);

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

    async fn unsubscribe_kline_stream(state: Arc<RwLock<MarketDataEngineState>>, params: UnsubscribeKlineStreamParams, event_publisher: EventPublisher, exchange_manager: Arc<Mutex<ExchangeManager>>) -> Result<(), String> {
        // let exchange = params.exchange.clone();
        // let mut state = state.write().await;
        // let exchange = state.exchanges.get_mut(&exchange).unwrap();
        // 检查是否已经注册
        let exchange_manager_guard = exchange_manager.lock().await;
        // 创建无限循环，只有当已注册时才退出
        loop {
            if exchange_manager_guard.is_registered(&params.exchange).await {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
        let exchange = exchange_manager_guard.get_exchange_ref(&params.exchange).await?;
        exchange.unsubscribe_kline_stream(&params.symbol, params.interval.clone(), params.frequency).await.unwrap();

        let request_id = params.request_id;
        let response_event = ResponseEvent::MarketDataEngine(MarketDataEngineResponse::UnsubscribeKlineStreamSuccess(UnsubscribeKlineStreamSuccessResponse {
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
            Exchange::Metatrader5 => {
                let mut mt5 = MetaTrader5::new(event_publisher);
                // 启动mt5服务器
                
                mt5.start_mt5_server(false).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                mt5.initialize_client().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                // mt5.login(23643, "HhazJ520!!!!", "EBCFinancialGroupKY-Demo", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");
                mt5.login(76898751, "HhazJ520....", "Exness-MT5Trial5", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");

                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let mut mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;
                mt5_exchange.connect_websocket().await?;
                tracing::info!("{}交易所注册成功!", exchange);
                let mut state = state.write().await;
                state.exchanges.insert(exchange.clone(), mt5_exchange);
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

    pub async fn get_kline_series(&self, exchange: Exchange, symbol: String, interval: KlineInterval, limit: Option<u32>) -> Result<(), String> {
        match exchange {
            Exchange::Binance => {
                let interval = BinanceKlineInterval::from(interval);
                let mut state = self.state.write().await;
                let binance = state.exchanges.get_mut(&exchange).unwrap();
                binance.get_kline_series(&symbol, interval.into(), limit).await.unwrap();
                Ok(())

            }
            _ => {
                return Err("不支持的交易所".to_string());
            }
        }

    }
    
}

