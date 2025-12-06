mod context;
mod event_listener;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use star_river_core::custom_type::{StrategyId, StrategyName};
use strategy_stats::strategy_stats::StrategyStatsAccessor;
use tokio::sync::{RwLock, watch};

use crate::{strategy_stats::context::BacktestStrategyStatsContext, virtual_trading_system::BacktestVts};

#[derive(Debug)]
pub struct BacktestStrategyStats {
    pub context: Arc<RwLock<BacktestStrategyStatsContext>>,
}

impl BacktestStrategyStats {
    pub fn new(
        strategy_id: StrategyId,
        strategy_name: StrategyName,
        strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
        vts: Arc<BacktestVts>,
    ) -> Self {
        let context = BacktestStrategyStatsContext::new(strategy_id, strategy_name, strategy_time_watch_rx, vts);
        Self {
            context: Arc::new(RwLock::new(context)),
        }
    }
}

impl StrategyStatsAccessor for BacktestStrategyStats {
    type Context = BacktestStrategyStatsContext;

    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        &self.context
    }
}
