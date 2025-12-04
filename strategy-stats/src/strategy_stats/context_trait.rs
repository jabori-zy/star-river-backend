use std::{fmt::Debug, sync::Arc};

use snafu::IntoError;
use star_river_core::custom_type::{StrategyId, StrategyName};
use tokio::sync::{RwLock, broadcast};
use tokio_util::sync::CancellationToken;

use super::{error::StrategyStatsError, metadata::StrategyStatsMetadata};
use crate::{event::StrategyStatsEvent, snapshot::StatsSnapshotHistory, strategy_stats::error::SendEventFailedSnafu};

pub trait StrategyStatsMetaDataExt: Debug + Send + Sync + 'static {
    fn metadata(&self) -> &StrategyStatsMetadata;
    fn metadata_mut(&mut self) -> &mut StrategyStatsMetadata;
}

pub trait StrategyStatsInfoExt: StrategyStatsMetaDataExt {
    fn strategy_id(&self) -> StrategyId {
        self.metadata().strategy_id
    }
    fn strategy_name(&self) -> &StrategyName {
        &self.metadata().strategy_name
    }

    fn cancel_token(&self) -> &CancellationToken {
        &self.metadata().cancel_token
    }

    fn asset_snapshot_history_mut(&mut self) -> &mut StatsSnapshotHistory {
        &mut self.metadata_mut().asset_snapshot_history
    }

    fn asset_snapshot_history(&self) -> &StatsSnapshotHistory {
        &self.metadata().asset_snapshot_history
    }
}

impl<Ctx> StrategyStatsInfoExt for Ctx where Ctx: StrategyStatsMetaDataExt {}

pub trait StrategyStatsCommunicationExt: StrategyStatsMetaDataExt + StrategyStatsInfoExt {
    fn strategy_stats_event_sender(&self) -> &broadcast::Sender<StrategyStatsEvent> {
        &self.metadata().strategy_stats_event_sender
    }
    fn strategy_stats_event_receiver(&self) -> broadcast::Receiver<StrategyStatsEvent> {
        self.metadata().strategy_stats_event_sender.subscribe()
    }

    fn send_event(&self, event: StrategyStatsEvent) -> Result<(), StrategyStatsError> {
        self.strategy_stats_event_sender().send(event).map_err(|e| {
            SendEventFailedSnafu {
                strategy_name: self.strategy_name().clone(),
            }
            .into_error(Arc::new(e))
        })?;
        Ok(())
    }
}

impl<Ctx> StrategyStatsCommunicationExt for Ctx where Ctx: StrategyStatsMetaDataExt + StrategyStatsInfoExt {}
