use crate::Engine;
use crate::EngineName;
use crate::account_engine::AccountEngine;
use crate::backtest_strategy_engine::StrategyEngine;
use crate::cache_engine::CacheEngine;
use crate::exchange_engine::ExchangeEngine;
use crate::indicator_engine::IndicatorEngine;
use crate::market_engine::MarketEngine;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

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
    pub async fn new(database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        // 缓存引擎
        let cache_engine = Arc::new(Mutex::new(CacheEngine::new()));

        // 交易所引擎
        let exchange_engine = Arc::new(Mutex::new(ExchangeEngine::new(database.clone())));

        // 市场引擎
        let market_engine = MarketEngine::new(exchange_engine.clone());

        // 指标引擎
        let indicator_engine = IndicatorEngine::new(heartbeat.clone(), cache_engine.clone());

        // 策略引擎
        let strategy_engine = StrategyEngine::new(database.clone(), exchange_engine.clone(), heartbeat.clone());

        // 账户引擎
        let account_engine = AccountEngine::new(exchange_engine.clone(), database.clone(), heartbeat.clone());

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
