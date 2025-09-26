use super::{
    BacktestNodeContextTrait, CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorCommand, EventCenterSingleton,
    GetIndicatorLookbackCmdPayload, GetIndicatorLookbackCommand, GetKlineDataCmdPayload, GetKlineDataCommand, IndicatorEngineCommand,
    IndicatorNodeContext, InitIndicatorDataCmdPayload, InitIndicatorDataCommand, Response,
};
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
    pub async fn calculate_indicator(&self) -> Result<bool, String> {
        let mut is_all_success = true;

        let kline_key = self.selected_kline_key.clone();
        let min_interval_symbols = self.get_min_interval_symbols_ref();

        // 如果当前IndicatorNode选择的kline_key不是最小周期交易对，则直接返回true
        if !min_interval_symbols.contains(&kline_key) {
            tracing::warn!("[{}] selected symbol is not min interval, skip", self.get_node_name());
            return Ok(true);
        }

        let strategy_id = self.get_strategy_id().clone();
        let node_id = self.get_node_id().clone();

        for (ind_key, _) in self.indicator_keys.iter() {
            let (resp_tx, resp_rx) = oneshot::channel();
            let payload = GetKlineDataCmdPayload::new(kline_key.clone(), None, None);

            // 获取所有K线
            let get_kline_series_cmd = GetKlineDataCommand::new(node_id.clone(), resp_tx, Some(payload));

            self.get_strategy_command_sender().send(get_kline_series_cmd.into()).await.unwrap();
            let response = resp_rx.await.unwrap();
            if response.is_success() {
                let (resp_tx, resp_rx) = oneshot::channel();
                let payload = CalculateHistoryIndicatorCmdPayload::new(
                    strategy_id.clone(),
                    node_id.clone(),
                    kline_key.clone().into(),
                    response.kline_series.clone(),
                    ind_key.indicator_config.clone(),
                );
                let cmd: IndicatorEngineCommand = CalculateHistoryIndicatorCommand::new(node_id.clone(), resp_tx, Some(payload)).into();

                EventCenterSingleton::send_command(cmd.into()).await.unwrap();
                let response = resp_rx.await.unwrap();
                if response.is_success() {
                    let (resp_tx, resp_rx) = oneshot::channel();
                    let payload = InitIndicatorDataCmdPayload::new(ind_key.clone(), response.indicators.clone());
                    let update_indicator_params = InitIndicatorDataCommand::new(node_id.clone(), resp_tx, Some(payload));
                    self.get_strategy_command_sender()
                        .send(update_indicator_params.into())
                        .await
                        .unwrap();
                    let response = resp_rx.await.unwrap();
                    if response.is_success() {
                        continue;
                    }
                } else {
                    is_all_success = false;
                    break;
                }
            }
        }
        Ok(is_all_success)
    }
}
