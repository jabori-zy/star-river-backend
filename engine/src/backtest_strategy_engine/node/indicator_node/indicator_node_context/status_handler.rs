use super::{
    BacktestNodeContextTrait, CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorCommand, EventCenterSingleton,
    GetIndicatorLookbackCmdPayload, GetIndicatorLookbackCommand, GetKlineDataCmdPayload, GetKlineDataCommand, IndicatorEngineCommand,
    IndicatorNodeContext, InitIndicatorDataCmdPayload, InitIndicatorDataCommand, Response,
};
use snafu::Report;
use star_river_core::error::engine_error::node_error::IndicatorNodeError;
use tokio::sync::oneshot;

impl IndicatorNodeContext {
    pub(crate) async fn init_indicator_lookback(&mut self) {
        for keys in self.indicator_keys.keys() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = GetIndicatorLookbackCmdPayload::new(self.get_strategy_id().clone(), self.get_node_id().clone(), keys.clone());
            let cmd: IndicatorEngineCommand = GetIndicatorLookbackCommand::new(self.get_node_id().clone(), resp_tx, Some(payload)).into();
            EventCenterSingleton::send_command(cmd.into()).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.is_success() {
                self.indicator_lookback.insert(keys.clone(), response.lookback);
            }
        }
        tracing::debug!("[{}] init indicator lookback complete.", self.get_node_id());
    }

    // 计算指标(一次性将指标全部计算完成)
    pub async fn calculate_indicator(&self) -> Result<(), IndicatorNodeError> {

        let kline_key = self.selected_kline_key.clone();
        let min_interval_symbols = self.get_min_interval_symbols_ref();

        // 如果当前IndicatorNode选择的kline_key不是最小周期交易对，则直接返回true(节点不需要计算指标)
        if !min_interval_symbols.contains(&kline_key) {
            tracing::warn!("[{}] selected symbol is not min interval, skip", self.get_node_name());
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
