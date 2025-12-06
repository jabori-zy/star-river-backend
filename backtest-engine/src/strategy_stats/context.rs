use std::sync::Arc;

use chrono::{DateTime, Utc};
use star_river_core::custom_type::{StrategyId, StrategyName};
use strategy_stats::{
    StatsSnapshot,
    event::StrategyStatsUpdatedEvent,
    strategy_stats::{
        StrategyStatsCommunicationExt, StrategyStatsInfoExt, StrategyStatsMetaDataExt, StrategyStatsMetadata, error::StrategyStatsError,
    },
};
use tokio::sync::watch;
use virtual_trading::{event::VtsEvent, vts_trait::VtsCtxAccessor};

use crate::virtual_trading_system::BacktestVts;

#[derive(Debug)]
pub struct BacktestStrategyStatsContext {
    metadata: StrategyStatsMetadata,
    pub(crate) strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
    pub(crate) vts: Arc<BacktestVts>,
}

impl BacktestStrategyStatsContext {
    pub fn new(
        strategy_id: StrategyId,
        strategy_name: StrategyName,
        strategy_time_watch_rx: watch::Receiver<DateTime<Utc>>,
        vts: Arc<BacktestVts>,
    ) -> Self {
        let metadata = StrategyStatsMetadata::new(strategy_id, strategy_name);
        Self {
            metadata,
            strategy_time_watch_rx,
            vts,
        }
    }
}

impl StrategyStatsMetaDataExt for BacktestStrategyStatsContext {
    fn metadata(&self) -> &StrategyStatsMetadata {
        &self.metadata
    }
    fn metadata_mut(&mut self) -> &mut StrategyStatsMetadata {
        &mut self.metadata
    }
}

impl BacktestStrategyStatsContext {
    fn current_time(&self) -> DateTime<Utc> {
        *self.strategy_time_watch_rx.borrow()
    }

    pub(crate) async fn handle_vts_event(&mut self, event: VtsEvent) -> Result<(), StrategyStatsError> {
        // Handle event and update asset snapshot
        match event {
            VtsEvent::UpdateFinished => {
                self.handle_vts_update_finished().await?;
            }
            _ => {}
        }

        Ok(())
    }

    pub async fn handle_vts_update_finished(&mut self) -> Result<(), StrategyStatsError> {
        let snapshot = self.create_snapshot().await;

        let stats_updated_event = StrategyStatsUpdatedEvent::new(self.strategy_id(), snapshot.clone(), self.current_time());
        self.send_event(stats_updated_event.into())?;
        // Add to history
        self.asset_snapshot_history_mut().add_snapshot(snapshot);
        Ok(())
    }

    async fn create_snapshot(&mut self) -> StatsSnapshot {
        // let datetime = trading_system.get_datetime(); // Timestamp
        let datetime = self.current_time();

        let (balance, initial_balance, available_balance, unrealized_pnl, equity, realized_pnl) = self
            .vts
            .with_ctx_read(|ctx| {
                let balance = ctx.balance;
                let initial_balance = ctx.initial_balance;
                let available_balance = ctx.available_balance;
                let unrealized_pnl = ctx.unrealized_pnl;
                let equity = ctx.equity;
                let realized_pnl = ctx.realized_pnl;
                (balance, initial_balance, available_balance, unrealized_pnl, equity, realized_pnl)
            })
            .await;

        // Create asset snapshot
        StatsSnapshot::new(
            datetime,
            initial_balance,
            balance,
            available_balance,
            unrealized_pnl,
            equity,
            realized_pnl,
        )
        // tracing::debug!("Strategy stats module created asset snapshot: equity={:.2}, cumulative_return={:.2}%, position_count={}",
        //     asset_snapshot_history_guard.get_latest_snapshot().unwrap().equity,
        //     asset_snapshot_history_guard.get_latest_snapshot().unwrap().cumulative_return,
        //     asset_snapshot_history_guard.get_latest_snapshot().unwrap().position_count);
    }

    pub fn clear_asset_snapshots(&mut self) {
        tracing::debug!("clear strategy stats snapshots");
        self.asset_snapshot_history_mut().clear();
    }
}
