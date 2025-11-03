// third-party
use tokio::sync::oneshot;

// workspace crate
use star_river_core::{
    custom_type::PlayIndex,
    key::{KeyTrait, key::KlineKey},
    market::Kline,
};
use event_center::communication::{
    Response,
    backtest_strategy::{GetKlineDataCmdPayload, GetKlineDataCommand},
};

// current crate
use super::{KlineNodeContext, KlineNodeError};
use crate::{
    error::node_error::kline_node_error::GetPlayKlineDataFailedSnafu,
    node::node_context_trait::{NodeCommunication, NodeIdentity},
};

impl KlineNodeContext {
    // 从策略中获取k线数据
    pub async fn get_kline_from_strategy(
        &self,
        kline_key: &KlineKey,
        play_index: PlayIndex, // 播放索引
    ) -> Result<Vec<Kline>, KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(kline_key.clone(), Some(play_index), Some(1));
        let get_kline_params = GetKlineDataCommand::new(self.node_id().clone(), resp_tx, Some(payload));

        self.strategy_command_sender().send(get_kline_params.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        if response.is_success() {
            return Ok(response.kline_series.clone());
        }
        Err(GetPlayKlineDataFailedSnafu {
            node_name: self.node_name().clone(),
            kline_key: kline_key.get_key_str(),
            play_index: play_index as u32,
        }
        .fail()?)
    }
}
