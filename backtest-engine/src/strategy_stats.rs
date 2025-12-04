mod context;
mod event_listener;

use std::sync::Arc;

use chrono::{DateTime, Utc};
use star_river_core::{
    custom_type::{Balance, StrategyId, StrategyName},
    system::DateTimeUtc,
};
use strategy_stats::strategy_stats::StrategyStatsAccessor;
use tokio::sync::{Mutex, RwLock, broadcast, watch};
use tokio_stream::{StreamExt, wrappers::BroadcastStream};
use tokio_util::sync::CancellationToken;
use virtual_trading::{Vts, context::VtsContext, event::VtsEvent};

use crate::{strategy_stats::context::BacktestStrategyStatsContext, virtual_trading_system::BacktestVts};

// T: VirtualTradingSystem

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
