use event_center::{communication::{engine::indicator_engine::{GetIndicatorLookbackCmdPayload, GetIndicatorLookbackCommand, IndicatorEngineCommand}, Response}, EventCenterSingleton};
use tokio::sync::oneshot;

use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;

use super::IndicatorNodeContext;


impl IndicatorNodeContext {
    pub(crate) async fn init_indicator_lookback(&mut self) {
        for keys in self.indicator_keys.keys() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = GetIndicatorLookbackCmdPayload::new(
                self.get_strategy_id().clone(),
                self.get_node_id().clone(),
                keys.clone(),
            );
            let cmd: IndicatorEngineCommand = GetIndicatorLookbackCommand::new(self.get_node_id().clone(), resp_tx, Some(payload)).into();
            EventCenterSingleton::send_command(cmd.into()).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.is_success() {
                self.indicator_lookback.insert(keys.clone(), response.lookback);
            }
        }
        tracing::debug!("[{}] init indicator lookback complete. lookback: {:#?}", self.get_node_id(), self.indicator_lookback);
    }
}