use key::{KeyTrait, KlineKey};
use snafu::{IntoError, OptionExt, ResultExt};
use star_river_core::kline::Kline;
use strategy_core::{
    communication::strategy::StrategyResponse,
    error::node_error::StrategyCmdRespRecvFailedSnafu,
    node::context_trait::{NodeCommunicationExt, NodeInfoExt},
};
// third-party
use tokio::sync::oneshot;

// current crate
use super::{KlineNodeContext, KlineNodeError};
use crate::{
    node::node_error::kline_node_error::{GetPlayKlineDataFailedSnafu, ReturnEmptyKlineSnafu},
    strategy::{
        PlayIndex,
        strategy_command::{GetKlineDataCmdPayload, GetKlineDataCommand},
    },
};

impl KlineNodeContext {
    // 从策略中获取k线数据
    pub async fn get_single_kline_from_strategy(
        &self,
        kline_key: &KlineKey,
        play_index: Option<PlayIndex>, // 播放索引
    ) -> Result<Kline, KlineNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(kline_key.clone(), play_index, Some(1));
        let get_kline_cmd = GetKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.send_strategy_command(get_kline_cmd.into()).await?;

        // 等待响应
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { payload, .. } => {
                let kline = payload.kline_series.first().context(ReturnEmptyKlineSnafu {
                    node_name: self.node_name().clone(),
                    kline_key: kline_key.key_str(),
                    play_index: play_index,
                })?;
                return Ok(kline.clone());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(GetPlayKlineDataFailedSnafu {
                    node_name: self.node_name().clone(),
                    kline_key: kline_key.key_str(),
                    play_index: play_index,
                }
                .into_error(error));
            }
        }
    }
}
