use snafu::IntoError;
// third-party
use tokio::sync::oneshot;




// current crate
use super::{KlineNodeContext, KlineNodeError};
use crate::{
    error::node_error::kline_node_error::GetPlayKlineDataFailedSnafu,
};
use key::{KeyTrait, KlineKey};
use star_river_core::custom_type::PlayIndex;
use star_river_core::kline::Kline;
use crate::strategy_command::{GetKlineDataCmdPayload, GetKlineDataCommand};
use strategy_core::node::context_trait::{NodeIdentityExt, NodeCommunicationExt};
use strategy_core::communication::strategy::StrategyResponse;

impl KlineNodeContext {
    // 从策略中获取k线数据
    pub async fn get_kline_from_strategy(
        &self,
        kline_key: &KlineKey,
        play_index: PlayIndex, // 播放索引
    ) -> Result<Vec<Kline>, KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(kline_key.clone(), Some(play_index), Some(1));
        let get_kline_params = GetKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);

        self.strategy_command_sender().send(get_kline_params.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        match response {
            StrategyResponse::Success { payload,.. } => {
                return Ok(payload.kline_series.clone());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(GetPlayKlineDataFailedSnafu {
                    node_name: self.node_name().clone(),
                    kline_key: kline_key.get_key_str(),
                    play_index: play_index as u32,
                }.into_error(error))
            }
        }
    }
}
