use event_center::EventCenterSingleton;
use event_center_core::communication::Response;
use key::IndicatorKey;
use snafu::{IntoError, ResultExt};
use star_river_core::kline::Kline;
use star_river_event::communication::{CalculateIndicatorCmdPayload, CalculateIndicatorCommand, IndicatorEngineCommand};
use strategy_core::{
    communication::strategy::StrategyResponse,
    error::node_error::{StrategyCmdRespRecvFailedSnafu, StrategySnafu},
    node::context_trait::{NodeCommunicationExt, NodeInfoExt},
};
use ta_lib::Indicator;
use tokio::sync::oneshot;

use super::IndicatorNodeContext;
use crate::{
    node::node_error::indicator_node_error::{CalculateIndicatorFailedSnafu, GetKlineDataFailedSnafu, IndicatorNodeError},
    strategy::strategy_command::{
        GetIndicatorDataCmdPayload, GetIndicatorDataCommand, GetKlineDataCmdPayload, GetKlineDataCommand, InitIndicatorDataCmdPayload,
        InitIndicatorDataCommand,
    },
};

impl IndicatorNodeContext {
    // 更新当前节点缓存的用于计算的k线数据
    pub(super) async fn update_kline_slice_cache(&mut self, indicator_key: IndicatorKey, kline_data: Kline) {
        // 如果指标缓存键不存在，则直接插入
        if !self.cache_kline_slice.contains_key(&indicator_key) {
            self.cache_kline_slice.insert(indicator_key.clone(), vec![kline_data]);
            return;
        }

        // 如果指标缓存键存在，则更新
        if let Some(kline_list) = self.cache_kline_slice.get_mut(&indicator_key) {
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
    pub(super) async fn get_indicator_from_strategy(
        &mut self,
        indicator_key: &IndicatorKey,
    ) -> Result<Option<Indicator>, IndicatorNodeError> {
        // Calculate index hint based on correct_index
        // If cycle_id is sequential (correct_index + 1), use cycle_id for fast path
        // Otherwise, use correct_index + 1 to recover from index drift
        let index = if self.cycle_id() == self.correct_index + 1 {
            Some(self.cycle_id()) // Sequential playback, use cycle_id directly
        } else {
            Some(self.correct_index + 1) // Index drift detected, use corrected value
        };

        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetIndicatorDataCmdPayload::new(indicator_key.clone(), Some(self.strategy_time()), index, Some(1));
        let get_indicator_cmd = GetIndicatorDataCommand::new(self.node_id().clone(), resp_tx, payload);

        self.send_strategy_command(get_indicator_cmd.into()).await?;

        // 等待响应
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { payload, .. } => {
                // Update correct_index if provided in response
                if let Some(correct_index) = payload.correct_index {
                    self.correct_index = correct_index;
                }
                return Ok(payload.indicator_series.first().cloned());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(StrategySnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error)
                .into());
            }
        }
    }

    pub(super) async fn get_kline_data_from_strategy(&self) -> Result<Vec<Kline>, IndicatorNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(self.selected_kline_key.clone(), None, Some(self.cycle_id()), None);
        // 获取所有K线
        let get_kline_series_cmd = GetKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);

        self.send_strategy_command(get_kline_series_cmd.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { payload, .. } => {
                return Ok(payload.kline_series.clone());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(GetKlineDataFailedSnafu {}.into_error(error));
            }
        }
    }

    pub(super) async fn calculate_single_indicator(
        &self,
        indicator_key: &IndicatorKey,
        kline_data: &Vec<Kline>,
    ) -> Result<Vec<Indicator>, IndicatorNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = CalculateIndicatorCmdPayload::new(
            self.strategy_id().clone(),
            self.node_id().clone(),
            indicator_key.get_kline_key(),
            kline_data.clone(),
            indicator_key.indicator_config.clone(),
        );
        let cmd: IndicatorEngineCommand = CalculateIndicatorCommand::new(self.node_id().clone(), resp_tx, payload).into();
        EventCenterSingleton::send_command(cmd.into()).await?;

        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            Response::Success { payload, .. } => {
                return Ok(payload.indicators.clone());
            }
            Response::Fail { error, .. } => {
                return Err(CalculateIndicatorFailedSnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error));
            }
        }
    }

    pub(super) async fn init_stragegy_indicator_data(
        &self,
        indicator_key: &IndicatorKey,
        indicators: &Vec<Indicator>,
    ) -> Result<(), IndicatorNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = InitIndicatorDataCmdPayload::new(indicator_key.clone(), indicators.clone());
        let cmd = InitIndicatorDataCommand::new(self.node_id().clone(), resp_tx, payload);
        self.send_strategy_command(cmd.into()).await?;
        let response = resp_rx.await.context(StrategyCmdRespRecvFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        match response {
            StrategyResponse::Success { .. } => {
                return Ok(());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(StrategySnafu {
                    node_name: self.node_name().clone(),
                }
                .into_error(error)
                .into());
            }
        }
    }
}
