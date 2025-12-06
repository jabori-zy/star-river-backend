use chrono::{DateTime, Utc};
use key::KlineKey;
use snafu::{IntoError, ResultExt};
use star_river_core::kline::Kline;
use strategy_core::{
    communication::strategy::StrategyResponse,
    error::node_error::{StrategyCmdRespRecvFailedSnafu, StrategySnafu},
    node::context_trait::{NodeCommunicationExt, NodeInfoExt},
};
// third-party
use tokio::sync::oneshot;

// current crate
use super::{KlineNodeContext, KlineNodeError};
use crate::strategy::strategy_command::{GetKlineDataCmdPayload, GetKlineDataCommand};

impl KlineNodeContext {
    // Get kline data from strategy
    pub async fn get_single_kline_from_strategy(
        &mut self,
        kline_key: &KlineKey,
        datetime: Option<DateTime<Utc>>,
    ) -> Result<Option<Kline>, KlineNodeError> {
        // Calculate index hint based on correct_index
        // If cycle_id is sequential (correct_index + 1), use cycle_id for fast path
        // Otherwise, use correct_index + 1 to recover from index drift
        let index = if self.cycle_id() == self.correct_index + 1 {
            Some(self.cycle_id()) // Sequential playback, use cycle_id directly
        } else {
            Some(self.correct_index + 1) // Index drift detected, use corrected value
        };

        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(kline_key.clone(), datetime, index, Some(1));
        let get_kline_cmd = GetKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.send_strategy_command(get_kline_cmd.into()).await?;

        // Wait for response
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { payload, .. } => {
                if let Some(correct_index) = payload.correct_index {
                    self.correct_index = correct_index;
                }
                return Ok(payload.kline_series.first().cloned());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(StrategySnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error)
                .into());
            }
        }
    }
}
