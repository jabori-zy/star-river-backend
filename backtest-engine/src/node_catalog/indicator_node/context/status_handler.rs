use super::IndicatorNodeContext;
use crate::node::node_error::indicator_node_error::IndicatorNodeError;
use star_river_event::communication::IndicatorEngineCommand;
use event_center::EventCenterSingleton;
use tokio::sync::oneshot;
use strategy_core::node::context_trait::NodeIdentityExt;
use star_river_event::communication::GetIndicatorLookbackCmdPayload;
use star_river_event::communication::GetIndicatorLookbackCommand;
use event_center_core::communication::Response;

impl IndicatorNodeContext {
    pub(crate) async fn init_indicator_lookback(&mut self) {
        for keys in self.indicator_keys.keys() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = GetIndicatorLookbackCmdPayload::new(self.strategy_id().clone(), self.node_id().clone(), keys.clone());
            let cmd: IndicatorEngineCommand = GetIndicatorLookbackCommand::new(self.node_id().clone(), resp_tx, payload).into();
            EventCenterSingleton::send_command(cmd.into()).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.is_success() {
                let lookback = response.into_payload().unwrap().lookback;
                self.indicator_lookback.insert(keys.clone(), lookback);
            }
        }
        tracing::debug!("[{}] init indicator lookback complete.", self.node_id());
    }

    // 计算指标(一次性将指标全部计算完成)
    pub async fn calculate_indicator(&self) -> Result<(), IndicatorNodeError> {

        let kline_key = self.selected_kline_key.clone();
        let min_interval_symbols = self.min_interval_symbols();

        // 如果当前IndicatorNode选择的kline_key不是最小周期交易对，则直接返回true(节点不需要计算指标)
        if !min_interval_symbols.contains(&kline_key) {
            tracing::warn!("[{}] selected symbol is not min interval, skip", self.node_name());
            return Ok(());
        }

        for (ind_key, _) in self.indicator_keys.iter() {
            let kline_data = self.get_kline_data().await?;
            let indicators = self.calculate_single_indicator(ind_key, &kline_data).await?;
            self.init_stragegy_indicator_data(ind_key, &indicators).await?;
        }
        Ok(())
    }
}
