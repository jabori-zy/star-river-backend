mod utils;
pub mod binance;
pub mod metatrader5;

use async_trait::async_trait;
use types::market::KlineInterval;
use std::any::Any;
use types::order::{OrderType, OrderSide, OrderRequest, Order};
use std::collections::HashMap;
use types::market::Exchange;
use tokio::sync::RwLock;
use tokio::sync::{RwLockReadGuard,RwLockWriteGuard};
use std::sync::Arc;
use event_center::EventPublisher;
use crate::binance::BinanceExchange;
use crate::metatrader5::MetaTrader5;
use event_center::Event;
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use event_center::command_event::CommandEvent;
use event_center::command_event::ExchangeManagerCommand;
use event_center::command_event::RegisterExchangeParams;
use event_center::response_event::ResponseEvent;
use event_center::response_event::ExchangeManagerResponse;
use event_center::response_event::RegisterExchangeSuccessResponse;
use utils::get_utc8_timestamp;

#[async_trait]
pub trait ExchangeClient: Send + Sync + Any {
    fn as_any(&self) -> &dyn Any;
    fn clone_box(&self) -> Box<dyn ExchangeClient>;
    async fn get_ticker_price(&self, symbol: &str) -> Result<serde_json::Value, String>;
    async fn get_kline_series(&self, symbol: &str, interval: KlineInterval, limit: Option<u32>) -> Result<(), String>;
    async fn connect_websocket(&mut self) -> Result<(), String>;
    async fn subscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String>;
    async fn unsubscribe_kline_stream(&self, symbol: &str, interval: KlineInterval, frequency: u32) -> Result<(), String>;
    async fn get_socket_stream(&self) -> Result<(), String>;
    async fn open_long(&mut self, order_type: OrderType, symbol: &str, quantity: f64, price: f64, tp: Option<f64>, sl: Option<f64>) -> Result<Order, String>; // 开多仓

}



pub struct ExchangeManager {
    pub exchanges: Arc<RwLock<HashMap<Exchange, Box<dyn ExchangeClient>>>>,
    pub event_publisher: EventPublisher,
    command_event_receiver: broadcast::Receiver<Event>,
    response_event_receiver: broadcast::Receiver<Event>,
}


impl ExchangeManager {
    pub fn new(event_publisher: EventPublisher, command_event_receiver: broadcast::Receiver<Event>, response_event_receiver: broadcast::Receiver<Event>) -> Self {
        Self {
            exchanges: Arc::new(RwLock::new(HashMap::new())),
            event_publisher,
            command_event_receiver,
            response_event_receiver,
        }
    }

    pub async fn start(&self) -> Result<(), String> {

        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        self.listen(internal_tx).await;
        self.handle_events(internal_rx, self.event_publisher.clone(), self.exchanges.clone()).await;

        Ok(())
    }

    async fn listen(&self, internal_tx: mpsc::Sender<Event>) {
        tracing::info!("交易所管理器启动成功, 开始监听...");
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
    }

    async fn handle_events(&self, mut internal_rx: mpsc::Receiver<Event>, event_publisher: EventPublisher, exchanges: Arc<RwLock<HashMap<Exchange, Box<dyn ExchangeClient>>>>) {
        tokio::spawn(async move {
            loop {
                let event = internal_rx.recv().await.unwrap();
                match event {
                    Event::Command(command_event) => {
                        Self::handle_command_event(command_event, event_publisher.clone(), exchanges.clone()).await;
                    }
                    _ => {}
                }
            }
        });
    }

    async fn handle_command_event(command_event: CommandEvent, event_publisher: EventPublisher, exchanges: Arc<RwLock<HashMap<Exchange, Box<dyn ExchangeClient>>>>) {
        match command_event {
            CommandEvent::ExchangeManager(exchange_manager_command) => {
                match exchange_manager_command {
                    ExchangeManagerCommand::RegisterExchange(register_exchange_command) => {
                        Self::register_exchange(register_exchange_command, event_publisher.clone(), exchanges.clone()).await.expect("注册交易所失败");
                    }
                }
            }
            _ => {}
        }
    }

    pub async fn register_exchange(register_params: RegisterExchangeParams, event_publisher: EventPublisher, exchanges: Arc<RwLock<HashMap<Exchange, Box<dyn ExchangeClient>>>>) -> Result<(), String> {
        // 检查是否已经注册
        let should_register = {
            let exchanges_guard = exchanges.read().await;
            !exchanges_guard.contains_key(&register_params.exchange)
        };
        if !should_register {
            // 直接发送响应事件
            tracing::warn!("{}交易所已注册, 无需重复注册", register_params.exchange);
            let response_event = ResponseEvent::ExchangeManager(ExchangeManagerResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
                exchange: register_params.exchange.clone(),
                response_timestamp: get_utc8_timestamp(),
                response_id: register_params.request_id,
            }));
            event_publisher.publish(response_event.clone().into()).unwrap();
            return Ok(());
        }
        
        match register_params.exchange {
            Exchange::Binance => {
                // 当类型为Box<dyn Trait Bound>时，需要显式地指定类型
                let mut binance_exchange = Box::new(BinanceExchange::new(event_publisher.clone())) as Box<dyn ExchangeClient>;
                binance_exchange.connect_websocket().await?;
                
                tracing::info!("{}交易所注册成功!", register_params.exchange);
                let mut state = exchanges.write().await;
                state.insert(register_params.exchange.clone(), binance_exchange);
                // 发送响应事件

                let response_event = ResponseEvent::ExchangeManager(ExchangeManagerResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
                    exchange: register_params.exchange.clone(),
                    response_timestamp: get_utc8_timestamp(),
                    response_id: register_params.request_id,
                }));
                event_publisher.publish(response_event.clone().into()).unwrap();
                
                Ok(())

            }
            Exchange::Metatrader5 => {
                let mut mt5 = MetaTrader5::new(event_publisher.clone());
                // 启动mt5服务器
                
                mt5.start_mt5_server(false).await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                mt5.initialize_client().await.unwrap();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                mt5.login(23643, "HhazJ520!!!!", "EBCFinancialGroupKY-Demo", r"C:\Program Files\MetaTrader 5\terminal64.exe").await.expect("登录失败");
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                
                let mut mt5_exchange = Box::new(mt5) as Box<dyn ExchangeClient>;
                mt5_exchange.connect_websocket().await?;
                tracing::info!("{}交易所注册成功!", register_params.exchange);
                let mut state = exchanges.write().await;
                state.insert(register_params.exchange.clone(), mt5_exchange);
                let response_event = ResponseEvent::ExchangeManager(ExchangeManagerResponse::RegisterExchangeSuccess(RegisterExchangeSuccessResponse {
                    exchange: register_params.exchange.clone(),
                    response_timestamp: get_utc8_timestamp(),
                    response_id: register_params.request_id,
                }));
                event_publisher.publish(response_event.clone().into()).unwrap();
                Ok(())
            }
            

            _ => {
                return Err("不支持的交易所".to_string());
            }
        }
    }

    pub async fn is_registered(&self, exchange: &Exchange) -> bool {
        let exchanges_guard = self.exchanges.read().await;
        exchanges_guard.contains_key(exchange)
    }

    pub async fn get_exchange(&self, exchange: &Exchange) -> Result<Box<dyn ExchangeClient>, String> {
        let exchanges_guard = self.exchanges.read().await;
        
        match exchanges_guard.get(exchange) {
            Some(client) => {
                // 使用clone_box方法直接获取一个新的Box<dyn ExchangeClient>
                Ok(client.clone_box())
            },
            None => Err(format!("交易所 {:?} 未注册", exchange))
        }
    }

    pub async fn get_exchange_ref<'a>(&'a self, exchange: &Exchange) -> Result<RwLockReadGuard<'a, Box<dyn ExchangeClient>>, String> {
        let exchanges_guard = self.exchanges.read().await;
        
        match exchanges_guard.get(exchange) {
            Some(_) => Ok(RwLockReadGuard::map(exchanges_guard, |guards| {
                guards.get(exchange).unwrap()
            })),
            None => Err(format!("交易所 {:?} 未注册", exchange))
        }
    }

    // 添加一个获取可变引用的方法
    pub async fn get_exchange_mut<'a>(&'a self, exchange: &Exchange) -> Result<tokio::sync::RwLockMappedWriteGuard<'a, Box<dyn ExchangeClient>>, String> {
        let exchanges_guard = self.exchanges.write().await;
        
        match exchanges_guard.get(exchange) {
            Some(_) => Ok(RwLockWriteGuard::map(exchanges_guard, |guards| {
                guards.get_mut(exchange).unwrap()
            })),
            None => Err(format!("交易所 {:?} 未注册", exchange))
        }
    }
}


