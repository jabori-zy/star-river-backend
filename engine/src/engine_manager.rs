use crate::Engine;
use crate::EngineName;
use crate::account_engine::AccountEngine;
use crate::backtest_strategy_engine::BacktestStrategyEngine;

use crate::exchange_engine::ExchangeEngine;
use crate::indicator_engine::IndicatorEngine;
use crate::market_engine::MarketEngine;
#[cfg(feature = "paid")]
use crate::live_strategy_engine::LiveStrategyEngine;
#[cfg(feature = "paid")]
use crate::cache_engine::CacheEngine;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct EngineManager {
    exchange_engine: Arc<Mutex<ExchangeEngine>>,
    market_engine: Arc<Mutex<MarketEngine>>,
    indicator_engine: Arc<Mutex<IndicatorEngine>>,
    strategy_engine: Arc<Mutex<BacktestStrategyEngine>>,
    account_engine: Arc<Mutex<AccountEngine>>,
    #[cfg(feature = "paid")]
    live_strategy_engine: Arc<Mutex<LiveStrategyEngine>>,
    #[cfg(feature = "paid")]
    cache_engine: Arc<Mutex<CacheEngine>>,
}

impl EngineManager {
    pub async fn new(database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        #[cfg(feature = "paid")]
        // 缓存引擎
        let cache_engine = Arc::new(Mutex::new(CacheEngine::new()));
        // 交易所引擎
        let exchange_engine = Arc::new(Mutex::new(ExchangeEngine::new(database.clone())));

        // 市场引擎
        let market_engine = MarketEngine::new(exchange_engine.clone());

        // 指标引擎
        let indicator_engine = IndicatorEngine::new(heartbeat.clone());

        // 策略引擎
        let strategy_engine = BacktestStrategyEngine::new(database.clone(), exchange_engine.clone(), heartbeat.clone());

        // 账户引擎
        let account_engine = AccountEngine::new(exchange_engine.clone(), database.clone(), heartbeat.clone());

        #[cfg(feature = "paid")]
        let live_strategy_engine = LiveStrategyEngine::new();

        Self {
            exchange_engine,
            market_engine: Arc::new(Mutex::new(market_engine)),
            indicator_engine: Arc::new(Mutex::new(indicator_engine)),
            strategy_engine: Arc::new(Mutex::new(strategy_engine)),
            account_engine: Arc::new(Mutex::new(account_engine)),
            #[cfg(feature = "paid")]
            cache_engine: cache_engine,
            #[cfg(feature = "paid")]
            live_strategy_engine: Arc::new(Mutex::new(live_strategy_engine)),
        }
    }

    pub async fn start(&self) {
        // 启动基础引擎
        self.start_basic_engines().await;
        
        // 启动付费功能引擎
        #[cfg(feature = "paid")]
        self.start_premium_engines().await;
    }

    async fn start_basic_engines(&self) {
        self.start_exchange_engine().await;
        self.start_market_engine().await;
        self.start_indicator_engine().await;
        self.start_strategy_engine().await;
        self.start_account_engine().await;
    }

    #[cfg(feature = "paid")]
    async fn start_premium_engines(&self) {
        self.start_cache_engine().await;
        // 如果有更多付费功能，可以在这里添加
        tracing::info!("付费功能引擎已启动");
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

    #[cfg(feature = "paid")]
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
            #[cfg(feature = "paid")]
            EngineName::CacheEngine => self.cache_engine.clone(),
            #[cfg(feature = "paid")]
            EngineName::LiveStrategyEngine => self.live_strategy_engine.clone(),
            EngineName::AccountEngine => self.account_engine.clone(),
        }
    }

    #[cfg(feature = "paid")]
    pub async fn get_cache_engine(&self) -> Arc<Mutex<CacheEngine>> {
        self.cache_engine.clone()
    }
}
