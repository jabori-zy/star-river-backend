pub mod backtest_strategy_manager;

use crate::EngineContext;
use crate::EngineName;
use crate::exchange_engine::ExchangeEngine;
use async_trait::async_trait;
use database::query::strategy_config_query::StrategyConfigQuery;
use event_center::event::Event;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::backtest_strategy_engine::strategy::BacktestStrategy;
use event_center::communication::engine::EngineCommand;
use snafu::{Report, ResultExt};
use star_river_core::custom_type::StrategyId;
use star_river_core::error::engine_error::strategy_engine_error::*;
use star_river_core::strategy::{StrategyConfig, TradeMode};

#[derive(Debug)]
pub struct StrategyEngineContext {
    pub engine_name: EngineName,
    pub database: DatabaseConnection,
    pub exchange_engine: Arc<Mutex<ExchangeEngine>>,
    pub heartbeat: Arc<Mutex<Heartbeat>>,
    // pub live_strategy_list: HashMap<StrategyId, LiveStrategy>,
    pub backtest_strategy_list: Arc<Mutex<HashMap<StrategyId, BacktestStrategy>>>,
    pub initializing_strategies: Arc<Mutex<HashSet<StrategyId>>>,
}

impl Clone for StrategyEngineContext {
    fn clone(&self) -> Self {
        Self {
            engine_name: self.engine_name.clone(),
            backtest_strategy_list: self.backtest_strategy_list.clone(),
            database: self.database.clone(),
            exchange_engine: self.exchange_engine.clone(),
            heartbeat: self.heartbeat.clone(),
            initializing_strategies: self.initializing_strategies.clone(),
        }
    }
}

#[async_trait]
impl EngineContext for StrategyEngineContext {
    fn clone_box(&self) -> Box<dyn EngineContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_engine_name(&self) -> EngineName {
        self.engine_name.clone()
    }

    async fn handle_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_command(&mut self, command: EngineCommand) {
        let _command = command;
    }
}

impl StrategyEngineContext {
    // pub async fn get_live_strategy_instance(&self, strategy_id: StrategyId) -> Result<&LiveStrategy, String> {
    //     if let Some(strategy) = self.live_strategy_list.get(&strategy_id) {
    //         Ok(strategy)
    //     } else {
    //         Err("策略不存在".to_string())
    //     }
    // }

    // pub async fn get_live_strategy_instance_mut(&mut self, strategy_id: StrategyId) -> Result<&mut LiveStrategy, String> {
    //     if let Some(strategy) = self.live_strategy_list.get_mut(&strategy_id) {
    //         Ok(strategy)
    //     } else {
    //         Err("策略不存在".to_string())
    //     }
    // }

    pub async fn get_backtest_strategy_instance(&self, strategy_id: StrategyId) -> Result<BacktestStrategy, StrategyEngineError> {
        let backtest_strategy_list = self.backtest_strategy_list.lock().await;
        if let Some(strategy) = backtest_strategy_list.get(&strategy_id) {
            Ok(strategy.clone())
        } else {
            let error = StrategyInstanceNotFoundSnafu { strategy_id }.build();
            let report = Report::from_error(&error);
            tracing::error!("{}", report);
            Err(error)
        }
    }

    // 注意：由于 backtest_strategy_list 是 Arc<Mutex<HashMap<...>>>，
    // 无法直接返回可变引用。如需修改策略，请考虑使用其他方法。
    // pub async fn get_backtest_strategy_instance_mut(&mut self, strategy_id: StrategyId) -> Result<&mut BacktestStrategy, String> {
    //     // 此方法无法实现，因为无法返回指向 Mutex 保护数据的可变引用
    // }

    pub async fn get_strategy_info_by_id(&self, id: i32) -> Result<StrategyConfig, StrategyEngineError> {
        let strategy = StrategyConfigQuery::get_strategy_by_id(&self.database, id)
            .await
            .context(StrategyConfigNotFoundSnafu { strategy_id: id })?;
        Ok(strategy)
    }

    pub async fn remove_strategy_instance(&mut self, trade_mode: TradeMode, strategy_id: i32) -> Result<(), StrategyEngineError> {
        match trade_mode {
            // TradeMode::Live => {
            //     self.live_strategy_list.remove(&strategy_id);
            //     tracing::info!("实盘策略实例已移除，策略id: {}", strategy_id);
            // }
            TradeMode::Backtest => {
                self.backtest_strategy_list.lock().await.remove(&strategy_id);
                tracing::info!("回测策略实例已移除，策略id: {}", strategy_id);
            }
            _ => {
                tracing::error!("不支持的交易模式: {}", trade_mode);
                return Err(UnsupportedTradeModeSnafu {
                    trade_mode: trade_mode.to_string(),
                }
                .build());
            }
        }
        Ok(())
    }

    // 实例化策略
    // pub async fn instantiate_strategy(&mut self, strategy: Strategy) -> Result<i32, String> {
    //     match strategy.trade_mode {
    //         TradeMode::Live => {
    //             let strategy_id = strategy.id;
    //             let strategy = LiveStrategy::new(
    //                 strategy,
    //                 self.event_publisher.clone(),
    //                 self.command_publisher.clone(),
    //                 self.command_receiver.clone(),
    //                 self.market_event_receiver.resubscribe(),
    //                 self.response_event_receiver.resubscribe(),
    //                 self.exchange_engine.clone(),
    //                 self.database.clone(),
    //                 self.heartbeat.clone()
    //             ).await;
    //             self.live_strategy_list.insert(strategy_id, strategy);
    //             Ok(strategy_id)
    //         }
    //         TradeMode::Backtest => {
    //             let strategy_id = strategy.id;
    //             let strategy = BacktestStrategy::new(
    //                 strategy,
    //                 self.event_publisher.clone(),
    //                 self.command_publisher.clone(),
    //                 self.command_receiver.clone(),
    //                 self.market_event_receiver.resubscribe(),
    //                 self.response_event_receiver.resubscribe(),
    //                 self.database.clone(),
    //                 self.heartbeat.clone()
    //             ).await;
    //             self.backtest_strategy_list.insert(strategy_id, strategy);
    //             Ok(strategy_id)
    //         }
    //         _ => {
    //             tracing::error!("不支持的策略类型: {}", strategy.trade_mode);
    //             Err("不支持的策略类型".to_string())
    //         }
    //     }
    // }

    // 初始化策略
    // pub async fn init_strategy(&mut self, strategy_id: i32) -> Result<(), String> {
    //     // 判断策略是否在实盘策略列表中,或者回测策略列表中
    //     if self.live_strategy_list.contains_key(&strategy_id) || self.backtest_strategy_list.contains_key(&strategy_id) {
    //         tracing::warn!("策略已存在, 不进行初始化");
    //         return Ok(());
    //     }

    //     let strategy_info = self.get_strategy_info_by_id(strategy_id).await?;
    //     // 加载策略（实例化策略）
    //     self.instantiate_strategy(strategy_info.clone()).await?;

    //     match strategy_info.trade_mode {
    //         TradeMode::Live => {
    //             let live_strategy_instance = self.live_strategy_list.get_mut(&strategy_id).unwrap();
    //             live_strategy_instance.init_strategy().await.unwrap();
    //             return Ok(());
    //         }
    //         TradeMode::Backtest => {
    //             let backtest_strategy_instance = self.backtest_strategy_list.get_mut(&strategy_id).unwrap();
    //             backtest_strategy_instance.init_strategy().await.unwrap();
    //             return Ok(());
    //         }
    //         _ => {
    //             tracing::error!("不支持的策略类型: {}", strategy_info.trade_mode);
    //             Err("不支持的策略类型".to_string())
    //         }
    //     }
    // }

    // 获取回测策略的缓存键
    // pub async fn get_strategy_cache_keys(&self, trade_mode: TradeMode, strategy_id: i32) -> Vec<CacheKey> {
    //     match trade_mode {
    //         TradeMode::Live => {
    //             let live_strategy = self.live_strategy_list.get(&strategy_id).unwrap();
    //             live_strategy.get_context().read().await.get_cache_keys().await
    //         }
    //         TradeMode::Backtest => {
    //             let backtest_strategy = self.backtest_strategy_list.get(&strategy_id).unwrap();
    //             backtest_strategy.get_context().read().await.get_cache_keys().await
    //         }
    //         _ => {
    //             tracing::error!("不支持的策略类型: {}", trade_mode);
    //             return vec![];
    //         }
    //     }
    // }
}
