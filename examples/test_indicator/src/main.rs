
use event_center::EventCenter;
use database::DatabaseManager;
use exchange_client::ExchangeClient;
use types::market::Exchange;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::EnvFilter;
use event_center::command::Command;
use engine::engine_manager::EngineManager;
use event_center::command::exchange_engine_command::RegisterExchangeParams;
use event_center::command::exchange_engine_command::ExchangeEngineCommand;
use heartbeat::Heartbeat;
use std::sync::Arc;
use tokio::sync::Mutex;
use event_center::command::market_engine_command::MarketEngineCommand;
use types::market::KlineInterval;
use types::engine::EngineName;
use tokio::sync::oneshot;
use event_center::command::market_engine_command::GetKlineHistoryParams;
use types::strategy::TimeRange;
use engine::cache_engine::cache_engine_context::CacheEngineContext;
use engine::indicator_engine::indicator_engine_context::IndicatorEngineContext;
use engine::indicator_engine::calculate::CalculateIndicatorFunction;
use types::cache::Key;
use types::indicator::{IndicatorConfig, MAType, PriceSource, IndicatorTrait};
use types::indicator::indicator_define::momentum::*;
use types::indicator::indicator_define::overlap::*;
use types::indicator::indicator_define::cycle::*;
use types::indicator::indicator_define::price_transform::*;
use types::indicator::indicator_define::volatility::*;
use types::indicator::indicator_define::volume::*;
use ordered_float::OrderedFloat;







#[tokio::main]
async fn main() {

    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        .with_env_filter(filter)
        // build but do not install the subscriber.
        .init();

    let mut event_center = EventCenter::new().init_channel().await;

    let database = DatabaseManager::new().await;

    // 初始化数据库
    let heartbeat = Arc::new(Mutex::new(Heartbeat::new(100)));

    let engine_manager = Arc::new(Mutex::new(EngineManager::new(
        &mut event_center,
        database.get_conn(),
        heartbeat.clone()
    ).await));

    

    // 启动心跳
    tokio::spawn(async move {
        let heartbeat = heartbeat.lock().await;
        heartbeat.start().await.unwrap();
    });

    // 启动市场引擎
    let engine_manager_clone = engine_manager.clone();
    tokio::spawn(async move {
        let engine_manager = engine_manager_clone.lock().await;
        engine_manager.start().await;
    });

    
    
    let command_publisher = event_center.get_command_publisher();

    let engine_manager_clone = engine_manager.clone();
    // 注册交易所
    tokio::spawn(async move {
        tracing::info!("{}: 开始注册交易所", "test");
        let (resp_tx, resp_rx) = oneshot::channel();
        let register_param = RegisterExchangeParams {
            account_id: 1,
            exchange: Exchange::Metatrader5("Exness-MT5Trial5".to_string()),
            sender: "test".to_string(),
            timestamp: 1111,
            responder: resp_tx,
        };

        let register_exchange_command = ExchangeEngineCommand::RegisterExchange(register_param);
        tracing::info!("{}注册交易所: {:?}", "test", register_exchange_command);
        command_publisher.send(register_exchange_command.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        tracing::info!("{}收到注册交易所响应: {:?}", "test", response);
        
        tokio::time::sleep(Duration::from_secs(3)).await;

        // 订阅K线
        let (resp_tx, resp_rx) = oneshot::channel();
        let params = GetKlineHistoryParams {
            strategy_id: 1,
            node_id: "test".to_string(),
            account_id: 1,
            exchange: Exchange::Metatrader5("Exness-MT5Trial5".to_string()),
            symbol: "BTCUSDm".to_string(),
            interval: KlineInterval::Hours1,
            time_range: TimeRange {
                start_date: chrono::NaiveDate::parse_from_str("2025-07-08", "%Y-%m-%d").unwrap(),
                end_date: chrono::NaiveDate::parse_from_str("2025-07-10", "%Y-%m-%d").unwrap(),
            },
            sender: "test".to_string(),
            timestamp: 1111,
            responder: resp_tx,
        };
        let command_event = Command::MarketEngine(MarketEngineCommand::GetKlineHistory(params));
        command_publisher.send(command_event.into()).await.unwrap();

        let response = resp_rx.await.unwrap();
        tracing::info!("获取K线历史响应: {:?}", response);


        let (cache_keys, cache_engine) = {
            let engine_manager_guard = engine_manager_clone.lock().await;
            let cache_engine = engine_manager_guard.get_engine(EngineName::CacheEngine).await;
            let cache_engine_guard = cache_engine.lock().await;
            let cache_engine_context = cache_engine_guard.get_context().read().await.clone();
            let cache_engine_context_guard = cache_engine_context.as_any().downcast_ref::<CacheEngineContext>().unwrap();
            let cache = cache_engine_context_guard.cache.read().await;
            let cache_keys = cache.keys().cloned().collect::<Vec<Key>>();

            let cache_engine = engine_manager_guard.get_cache_engine().await;
            (cache_keys, cache_engine)
        };
        
        tracing::info!("缓存键: {:?}", cache_keys);

        let ma_config = IndicatorConfig::MA(MAConfig {
            time_period: 9,
            ma_type: MAType::SMA,
            price_source: PriceSource::Close,
        });

        let macd_config = IndicatorConfig::MACD(MACDConfig {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            price_source: PriceSource::Close,
        });

        let bbands_config = IndicatorConfig::BBands(BBandsConfig {
            time_period: 20,
            dev_up: OrderedFloat(2.0),
            dev_down: OrderedFloat(2.0),
            ma_type: MAType::SMA,
            price_source: PriceSource::Close,
        });

        //计算指标
        let result = CalculateIndicatorFunction::calculate_indicator(
            cache_engine.clone(),
            cache_keys[0].clone(),
            bbands_config,
            false,
        ).await;
        
        match result {
            Ok(indicators) => {
                let json = indicators.iter().map(|indicator| indicator.to_json()).collect::<Vec<serde_json::Value>>();
                let json_str = serde_json::to_string(&json).unwrap();
                tracing::info!("指标计算结果: {:}", json_str);
            },
            Err(e) => {
                tracing::error!("指标计算失败: {}", e);
            }
        }
        
    });
    tokio::time::sleep(tokio::time::Duration::from_secs(1000)).await;

}



