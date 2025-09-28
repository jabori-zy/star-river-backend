use super::{
    BacktestNodeContextTrait, GetKlineDataCmdPayload, GetKlineDataCommand, GetPlayKlineDataFailedSnafu, KeyTrait, Kline, KlineKey, KlineNodeContext,
    KlineNodeError, PlayIndex, Response,
};
use tokio::sync::oneshot;

impl KlineNodeContext {
    // 从策略中获取k线数据
    pub async fn get_kline(
        &self,
        kline_key: &KlineKey,
        play_index: PlayIndex, // 播放索引
    ) -> Result<Vec<Kline>, KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(kline_key.clone(), Some(play_index), Some(1));
        let get_kline_params = GetKlineDataCommand::new(self.get_node_id().clone(), resp_tx, Some(payload));

        self.get_strategy_command_sender().send(get_kline_params.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(response.kline_series.clone());
        }
        Err(GetPlayKlineDataFailedSnafu {
            node_name: self.get_node_name().clone(),
            kline_key: kline_key.get_key_str(),
            play_index: play_index as u32,
        }
        .fail()?)
    }
}
