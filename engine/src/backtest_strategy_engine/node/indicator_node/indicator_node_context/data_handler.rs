use super::{
    BacktestNodeContextTrait, GetIndicatorDataCmdPayload, GetIndicatorDataCommand, Indicator, IndicatorKey, IndicatorNodeContext, Kline,
    QuantData, Response,
};
use tokio::sync::oneshot;

impl IndicatorNodeContext {
    // 更新当前节点缓存的用于计算的k线数据
    pub(super) async fn update_kline_data(&mut self, indicator_key: IndicatorKey, kline_data: Kline) {
        // 如果指标缓存键不存在，则直接插入
        if !self.kline_value.contains_key(&indicator_key) {
            self.kline_value.insert(indicator_key.clone(), vec![kline_data]);
            return;
        }

        // 如果指标缓存键存在，则更新
        if let Some(kline_list) = self.kline_value.get_mut(&indicator_key) {
            if let Some(last_kline) = kline_list.last() {
                // 如果最新的一条数据时间戳等于最后一根k线的时间戳，则更新最后一条k线
                if last_kline.get_datetime() == kline_data.get_datetime() {
                    kline_list.pop();
                    kline_list.push(kline_data);
                } else {
                    // 如果最新的一条数据时间戳不等于最后一根k线的时间戳，则插入新数据
                    kline_list.push(kline_data);

                    // 检查是否需要限制长度
                    if let Some(lookback) = self.indicator_lookback.get(&indicator_key) {
                        if kline_list.len() > *lookback + 1 {
                            kline_list.remove(0);
                        }
                    }
                }
            } else {
                // 如果列表为空，直接插入
                kline_list.push(kline_data);
            }
        }
    }

    // 获取已经计算好的回测指标数据
    pub(super) async fn get_indicator_data(&self, indicator_key: &IndicatorKey, play_index: i32) -> Result<Indicator, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetIndicatorDataCmdPayload::new(indicator_key.clone(), Some(play_index), Some(1));
        let get_indicator_cmd = GetIndicatorDataCommand::new(self.get_node_id().clone(), resp_tx, Some(payload));

        self.get_strategy_command_sender().send(get_indicator_cmd.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            tracing::info!("indicator series: {:?}", response.indicator_series);
            return Ok(response.indicator_series.last().unwrap().clone());
        } else {
            return Err(format!("节点{}收到回测K线缓存数据失败", self.base_context.node_id));
        }
    }
}
