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
        // 缓存引擎
        // let cache_engine = Arc::new(Mutex::new(CacheEngine::new()));
        // 交易所引擎
        let exchange_engine = Arc::new(Mutex::new(ExchangeEngine::new(database.clone())));

        // 市场引擎
        let market_engine = MarketEngine::new(exchange_engine.clone());

        // 指标引擎
        let indicator_engine = IndicatorEngine::new();

        // 策略引擎
        // let strategy_engine = BacktestStrategyEngine::new(database.clone(), heartbeat.clone());
        let strategy_engine = BacktestEngine::new(database.clone(), heartbeat.clone());

        // 账户引擎
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
        // 启动基础引擎
        self.start_engines().await;

        // 启动付费功能引擎

        // self.start_premium_engines().await;
    }

    async fn start_engines(&self) {
        self.start_exchange_engine().await;
        self.start_market_engine().await;
        self.start_indicator_engine().await;
        self.start_backtest_engine().await;
        // self.start_account_engine().await;
    }

    // 启动交易所引擎并等待完成
    async fn start_exchange_engine(&self) {
        let engine = self.exchange_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
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
    async fn start_backtest_engine(&self) {
        let engine = self.backtest_engine.clone();
        tokio::spawn(async move {
            let engine = engine.lock().await;
            engine.start().await
        });
    }

    // 启动缓存引擎并等待完成
    // async fn start_cache_engine(&self) {
    //     let engine = self.cache_engine.clone();
    //     tokio::spawn(async move {
    //         let engine = engine.lock().await;
    //         engine.start().await
    //     });
    // }

    // 启动账户引擎
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
