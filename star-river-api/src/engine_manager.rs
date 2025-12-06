// use engine::account_engine::AccountEngine;
// use engine::backtest_strategy_engine::BacktestStrategyEngine;
// use engine::backtest_engine::BacktestEngine;

// use engine::cache_engine::CacheEngine;
// use engine::exchange_engine::ExchangeEngine;
// use engine::indicator_engine::IndicatorEngine;

// use engine::live_strategy_engine::LiveStrategyEngine;
// use engine::market_engine::MarketEngine;
use std::sync::Arc;

// use indicator_engine::IndicatorEngine;
use backtest_engine::BacktestEngine;
use engine_core::engine_trait::EngineLifecycle;
use exchange_engine::ExchangeEngine;
use heartbeat::Heartbeat;
use indicator_engine::IndicatorEngine;
use market_engine::MarketEngine;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct EngineManager {
    exchange_engine: Arc<Mutex<ExchangeEngine>>,
    market_engine: Arc<Mutex<MarketEngine>>,
    indicator_engine: Arc<Mutex<IndicatorEngine>>,
    // strategy_engine: Arc<Mutex<BacktestStrategyEngine>>,
    backtest_engine: Arc<Mutex<BacktestEngine>>,
    // account_engine: Arc<Mutex<AccountEngine>>,

    // live_strategy_engine: Arc<Mutex<LiveStrategyEngine>>,

    // cache_engine: Arc<Mutex<CacheEngine>>,
    // new_exchange_engine: Arc<Mutex<NewExchangeEngine>>,
}

impl EngineManager {
    pub async fn new(database: DatabaseConnection, heartbeat: Arc<Mutex<Heartbeat>>) -> Self {
        // Cache engine
        // let cache_engine = Arc::new(Mutex::new(CacheEngine::new()));
        // Exchange engine
        let exchange_engine = Arc::new(Mutex::new(ExchangeEngine::new(database.clone())));

        // Market engine
        let market_engine = MarketEngine::new(exchange_engine.clone());

        // Indicator engine
        let indicator_engine = IndicatorEngine::new();

        // Strategy engine
        // let strategy_engine = BacktestStrategyEngine::new(database.clone(), heartbeat.clone());
        let strategy_engine = BacktestEngine::new(database.clone(), heartbeat.clone());

        // Account engine
        // let account_engine = AccountEngine::new(exchange_engine.clone(), database.clone(), heartbeat.clone());

        // let live_strategy_engine = LiveStrategyEngine::new();

        Self {
            exchange_engine,
            market_engine: Arc::new(Mutex::new(market_engine)),
            indicator_engine: Arc::new(Mutex::new(indicator_engine)),
            backtest_engine: Arc::new(Mutex::new(strategy_engine)),
        }
    }

    pub async fn start(&self) {
        // Start basic engines
        self.start_engines().await;

        // Start premium feature engines

        // self.start_premium_engines().await;
    }

    async fn start_engines(&self) {
        self.start_exchange_engine().await;
        self.start_market_engine().await;
        self.start_indicator_engine().await;
        self.start_backtest_engine().await;
        // self.start_account_engine().await;
    }

    // Start exchange engine and wait for completion
    async fn start_exchange_engine(&self) {
        let engine = self.exchange_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }

    // Start market engine and wait for completion
    async fn start_market_engine(&self) {
        let engine = self.market_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }

    // Start indicator engine and wait for completion
    async fn start_indicator_engine(&self) {
        let engine = self.indicator_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }

    // Start strategy engine and wait for completion
    async fn start_backtest_engine(&self) {
        let engine = self.backtest_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }

    // Start cache engine and wait for completion
    // async fn start_cache_engine(&self) {
    //     let engine = self.cache_engine.clone();
    //     tokio::spawn(async move {
    //         let engine = engine.lock().await;
    //         engine.start().await
    //     });
    // }

    // Start account engine
    // async fn start_account_engine(&self) {
    //     let engine = self.account_engine.clone();
    //     tokio::spawn(async move {
    //         let engine = engine.lock().await;
    //         engine.start().await
    //     });
    // }

    pub async fn backtest_engine(&self) -> &Arc<Mutex<BacktestEngine>> {
        &self.backtest_engine
    }

    pub async fn market_engine(&self) -> &Arc<Mutex<MarketEngine>> {
        &self.market_engine
    }

    pub async fn indicator_engine(&self) -> &Arc<Mutex<IndicatorEngine>> {
        &self.indicator_engine
    }

    pub async fn exchange_engine(&self) -> &Arc<Mutex<ExchangeEngine>> {
        &self.exchange_engine
    }
    // pub async fn get_cache_engine(&self) -> Arc<Mutex<CacheEngine>> {
    //     self.cache_engine.clone()
    // }
}
