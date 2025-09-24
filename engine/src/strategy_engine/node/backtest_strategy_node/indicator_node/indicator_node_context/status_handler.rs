use event_center::{communication::engine::{indicator_engine::{GetIndicatorLookbackParams, IndicatorEngineResponse}, EngineResponse}, EventCenterSingleton};
use tokio::sync::oneshot;

use crate::strategy_engine::node::node_context::BacktestNodeContextTrait;

use super::IndicatorNodeContext;


impl IndicatorNodeContext {
    pub(crate) async fn init_indicator_lookback(&mut self) {
        for keys in self.indicator_keys.keys() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let get_indicator_lookback_params = GetIndicatorLookbackParams::new(
                self.get_strategy_id().clone(),
                self.get_node_id().clone(),
                keys.clone(),
                self.get_node_id().clone(),
                resp_tx,
            );
            EventCenterSingleton::send_command(get_indicator_lookback_params.into()).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.success() {
                match response {
                    EngineResponse::IndicatorEngine(IndicatorEngineResponse::GetIndicatorLookback(resp)) => {
                        self.indicator_lookback.insert(keys.clone(), resp.lookback);
                    }
                    _ => {}
                }
            }
        }
        tracing::debug!("[{}] init indicator lookback complete. lookback: {:#?}", self.get_node_id(), self.indicator_lookback);
    }
}