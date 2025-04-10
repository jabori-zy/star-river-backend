use std::sync::Arc;
use database::entities::order;
use event_center::EventPublisher;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use tokio::sync::broadcast;
use event_center::Event;
use types::market::Exchange;
use std::time::Duration;
use tokio::time::timeout;

use crate::exchange_engine;
use crate::indicator_engine::IndicatorEngine;
use crate::order_engine::OrderEngine;
use crate::Engine;
use crate::market_engine::MarketEngine;
use crate::exchange_engine::ExchangeEngine;
use crate::strategy_engine::StrategyEngine;
use crate::cache_engine::CacheEngine;
use crate::EngineName;

pub struct EngineManager {
    exchange_engine: Arc<Mutex<ExchangeEngine>>,
    market_engine: Arc<Mutex<MarketEngine>>,
    indicator_engine: Arc<Mutex<IndicatorEngine>>,
    order_engine: Arc<Mutex<OrderEngine>>,
    strategy_engine: Arc<Mutex<StrategyEngine>>,
    cache_engine: Arc<Mutex<CacheEngine>>,
}

impl EngineManager {
    pub fn new(
        event_publisher: EventPublisher,
        exchange_event_receiver: broadcast::Receiver<Event>,
        market_event_receiver: broadcast::Receiver<Event>,
        request_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        database: DatabaseConnection
    ) -> Self
    
    {
        let exchange_engine = Arc::new(Mutex::new(ExchangeEngine::new(
            event_publisher.clone(), 
            request_event_receiver.resubscribe(), 
            response_event_receiver.resubscribe())));

        // 新建市场引擎
        let market_engine = MarketEngine::new(
            event_publisher.clone(), 
            request_event_receiver.resubscribe(), 
            response_event_receiver.resubscribe(),
            exchange_engine.clone()
        );
        
        // 指标引擎
        let indicator_engine = IndicatorEngine::new(
            event_publisher.clone(), 
            request_event_receiver.resubscribe(), 
            response_event_receiver.resubscribe(),
        );

        let order_engine = OrderEngine::new(
            event_publisher.clone(), 
            request_event_receiver.resubscribe(), 
            response_event_receiver.resubscribe(), 
            exchange_engine.clone(), 
            database.clone());

        let strategy_engine = StrategyEngine::new(
            event_publisher.clone(), 
            market_event_receiver.resubscribe(), 
            request_event_receiver.resubscribe(), 
            response_event_receiver.resubscribe(), 
            database.clone());

        let cache_engine = CacheEngine::new(
            event_publisher.clone(), 
            exchange_event_receiver.resubscribe(),
            request_event_receiver.resubscribe(), 
            response_event_receiver.resubscribe(),
        );


        Self {
            exchange_engine,
            market_engine: Arc::new(Mutex::new(market_engine)),
            indicator_engine: Arc::new(Mutex::new(indicator_engine)),
            order_engine:Arc::new(Mutex::new(order_engine)),
            strategy_engine: Arc::new(Mutex::new(strategy_engine)),
            cache_engine: Arc::new(Mutex::new(cache_engine)),
        }
    }

    pub async fn start(&self) {
        self.start_exchange_engine().await;
        self.start_market_engine().await;
        self.start_indicator_engine().await;
        self.start_order_engine().await;
        self.start_strategy_engine().await;
        self.start_cache_engine().await;
    }
    
    // 启动交易所引擎并等待完成
    async fn start_exchange_engine(&self) {
        let engine = self.exchange_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await;
        });
    }
    
    // 启动市场引擎并等待完成
    async fn start_market_engine(&self) {
        let engine = self.market_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }
    
    // 启动指标引擎并等待完成
    async fn start_indicator_engine(&self) {
        let engine = self.indicator_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }
    
    // 启动订单引擎并等待完成
    async fn start_order_engine(&self) {
        let engine = self.order_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }
    
    // 启动策略引擎并等待完成
    async fn start_strategy_engine(&self) {
        let engine = self.strategy_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }

    // 启动缓存引擎并等待完成
    async fn start_cache_engine(&self) {
        let engine = self.cache_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }

    pub async fn get_engine(&self, engine_name: EngineName) -> Arc<Mutex<dyn Engine>> {
        match engine_name {
            EngineName::ExchangeEngine => self.exchange_engine.clone(),
            EngineName::MarketEngine => self.market_engine.clone(),
            EngineName::IndicatorEngine => self.indicator_engine.clone(),
            EngineName::OrderEngine => self.order_engine.clone(),
            EngineName::StrategyEngine => self.strategy_engine.clone(),
            EngineName::CacheEngine => self.cache_engine.clone(),
        }
    }

}