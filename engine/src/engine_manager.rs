use std::sync::Arc;
use event_center::EventPublisher;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use tokio::sync::{broadcast, mpsc};
use event_center::Event;

use crate::indicator_engine::IndicatorEngine;
use crate::Engine;
use crate::market_engine::MarketEngine;
use crate::exchange_engine::ExchangeEngine;
use crate::strategy_engine::StrategyEngine;
use crate::cache_engine::CacheEngine;
use crate::account_engine::AccountEngine;
use crate::EngineName;
use heartbeat::Heartbeat;
use event_center::EventCenter;
use event_center::command::Command;
use event_center::EventReceiver;
use event_center::Channel;


#[derive(Debug)]
pub struct EngineManager {
    exchange_engine: Arc<Mutex<ExchangeEngine>>,
    market_engine: Arc<Mutex<MarketEngine>>,
    indicator_engine: Arc<Mutex<IndicatorEngine>>,
    strategy_engine: Arc<Mutex<StrategyEngine>>,
    cache_engine: Arc<Mutex<CacheEngine>>,
    account_engine: Arc<Mutex<AccountEngine>>,
}

impl EngineManager {
    pub async fn new(
        event_center: &mut EventCenter,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>
    ) -> Self
    
    {

        let (cache_command_tx, cache_command_rx) = mpsc::channel::<Command>(100);
        let exchange_event_receiver = event_center.subscribe(&Channel::Exchange).await.unwrap();
        let cache_engine = Arc::new(Mutex::new(CacheEngine::new(
            event_center.get_event_publisher(),
            event_center.get_command_publisher(),
            cache_command_rx,
            exchange_event_receiver,
        )));

        // 交易所引擎
        let (exchange_command_tx, exchange_command_rx) = mpsc::channel::<Command>(100);
        let exchange_engine = Arc::new(Mutex::new(ExchangeEngine::new(
            event_center.get_event_publisher(), 
            event_center.get_command_publisher(),
            exchange_command_rx,
            database.clone()
        )));

        // 市场引擎
        let (market_command_tx, market_command_rx) = mpsc::channel::<Command>(100);
        let market_engine = MarketEngine::new(
            event_center.get_event_publisher(),
            event_center.get_command_publisher(),
            market_command_rx,
            exchange_engine.clone(),
            
        );
        
        // 指标引擎
        let (indicator_command_tx, indicator_command_rx) = mpsc::channel::<Command>(100);
        let exchange_event_receiver = event_center.subscribe(&Channel::Exchange).await.unwrap();
        let indicator_engine = IndicatorEngine::new(
            heartbeat.clone(),
            cache_engine.clone(),
            event_center.get_event_publisher(), 
            event_center.get_command_publisher(),
            indicator_command_rx,
            exchange_event_receiver,
        );

        // 策略引擎
        let (strategy_command_tx, strategy_command_rx) = mpsc::channel::<Command>(100);
        let market_event_receiver = event_center.subscribe(&Channel::Market).await.unwrap();

        let strategy_engine = StrategyEngine::new(
            event_center.get_event_publisher(), 
            event_center.get_command_publisher(),
            strategy_command_rx,
            market_event_receiver.resubscribe(),
            market_event_receiver.resubscribe(),
            market_event_receiver.resubscribe(),

            database.clone(),
            exchange_engine.clone(),
            heartbeat.clone()
        );

        
        // 账户引擎
        let (account_command_tx, account_command_rx) = mpsc::channel::<Command>(100);
        let account_event_receiver = event_center.subscribe(&Channel::Account).await.unwrap();
        let account_engine = AccountEngine::new(
            event_center.get_event_publisher(),
            event_center.get_command_publisher(),
            account_command_rx,
            account_event_receiver,
            exchange_engine.clone(),
            database.clone(),
            heartbeat.clone()
        );

        // 设置每一个引擎的命令发送器
        {
            event_center.set_engine_command_sender(EngineName::CacheEngine, cache_command_tx).await;
            event_center.set_engine_command_sender(EngineName::ExchangeEngine, exchange_command_tx).await;
            event_center.set_engine_command_sender(EngineName::MarketEngine, market_command_tx).await;
            event_center.set_engine_command_sender(EngineName::IndicatorEngine, indicator_command_tx).await;
            event_center.set_engine_command_sender(EngineName::StrategyEngine, strategy_command_tx).await;
            event_center.set_engine_command_sender(EngineName::AccountEngine, account_command_tx).await;
        }


        Self {
            exchange_engine,
            market_engine: Arc::new(Mutex::new(market_engine)),
            indicator_engine: Arc::new(Mutex::new(indicator_engine)),
            strategy_engine: Arc::new(Mutex::new(strategy_engine)),
            cache_engine: cache_engine,
            account_engine: Arc::new(Mutex::new(account_engine)),
        }
    }

    pub async fn start(&self) {
        self.start_exchange_engine().await;
        self.start_market_engine().await;
        self.start_indicator_engine().await;
        self.start_strategy_engine().await;
        self.start_cache_engine().await;
        self.start_account_engine().await;
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

    // 启动账户引擎
    async fn start_account_engine(&self) {
        let engine = self.account_engine.clone();
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
            EngineName::StrategyEngine => self.strategy_engine.clone(),
            EngineName::CacheEngine => self.cache_engine.clone(),
            EngineName::AccountEngine => self.account_engine.clone(),
        }
    }

    pub async fn get_cache_engine(&self) -> Arc<Mutex<CacheEngine>> {
        self.cache_engine.clone()
    }



}