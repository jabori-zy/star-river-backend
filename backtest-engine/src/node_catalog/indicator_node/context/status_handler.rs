use event_center::{CmdRespRecvFailedSnafu, EventCenterSingleton};
use event_center_core::communication::Response;
use key::KeyTrait;
use snafu::{IntoError, ResultExt};
use star_river_event::communication::{CalculateLookbackCmdPayload, CalculateLookbackCommand, IndicatorEngineCommand};
use strategy_core::node::context_trait::NodeInfoExt;
use tokio::sync::oneshot;

use super::IndicatorNodeContext;
use crate::node::node_error::indicator_node_error::{IndicatorEngineSnafu, IndicatorNodeError};

impl IndicatorNodeContext {
    pub(crate) async fn init_indicator_lookback(&mut self) -> Result<(), IndicatorNodeError> {
        for keys in self.indicator_keys.keys() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = CalculateLookbackCmdPayload::new(self.strategy_id().clone(), self.node_id().clone(), keys.clone());
            let cmd: IndicatorEngineCommand = CalculateLookbackCommand::new(self.node_id().clone(), resp_tx, payload).into();
            EventCenterSingleton::send_command(cmd.into()).await?;
            let response = resp_rx.await.context(CmdRespRecvFailedSnafu {})?;
            match response {
                Response::Success { payload, .. } => {
                    let lookback = payload.lookback;
                    self.indicator_lookback.insert(keys.clone(), lookback);
                }
                Response::Fail { error, .. } => {
                    return Err(IndicatorEngineSnafu {
                        node_name: self.node_name().clone(),
                    }
                    .into_error(error));
                }
            }
        }
        Ok(())
    }

    // Calculate indicators (complete all indicator calculations at once)
    pub async fn calculate_indicator(&self) -> Result<(), IndicatorNodeError> {
        let kline_key = self.selected_kline_key.clone();

        // If current IndicatorNode's selected kline_key is not the minimum interval trading pair, return directly (node doesn't need to calculate indicators)
        if self.min_interval != kline_key.interval() {
            return Ok(());
        }

        for (ind_key, _) in self.indicator_keys.iter() {
            let kline_data = self.get_kline_data_from_strategy().await?;
            let indicators = self.calculate_single_indicator(ind_key, &kline_data).await?;
            self.init_stragegy_indicator_data(ind_key, &indicators).await?;
        }
        Ok(())
    }
}
