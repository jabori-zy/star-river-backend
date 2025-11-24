use event_center::EventCenterSingleton;
use event_center_core::communication::Response;
use key::{IndicatorKey, KeyTrait, KlineKey};
use snafu::IntoError;
use star_river_core::kline::Kline;
use star_river_event::communication::{CalculateHistoryIndicatorCmdPayload, CalculateHistoryIndicatorCommand, IndicatorEngineCommand};
use strategy_core::{
    communication::strategy::StrategyResponse,
    node::context_trait::{NodeCommunicationExt, NodeInfoExt},
};
use ta_lib::{Indicator, IndicatorConfig};
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
    pub(super) async fn get_indicator_data(&self, indicator_key: &IndicatorKey) -> Result<Indicator, String> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetIndicatorDataCmdPayload::new(indicator_key.clone(), Some(self.cycle_id() as i32), Some(1));
        let get_indicator_cmd = GetIndicatorDataCommand::new(self.node_id().clone(), resp_tx, payload);

        self.send_strategy_command(get_indicator_cmd.into()).await.unwrap();

        // 等待响应
        let response = resp_rx.await.unwrap();
        match response {
            StrategyResponse::Success { payload, .. } => {
                return Ok(payload.indicator_series.last().unwrap().clone());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(format!("节点{}收到回测K线缓存数据失败", self.node_name()));
            }
        }
    }

    pub(super) async fn get_kline_data(&self) -> Result<Vec<Kline>, IndicatorNodeError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let payload = GetKlineDataCmdPayload::new(self.selected_kline_key.clone(), None, None);
        // 获取所有K线
        let get_kline_series_cmd = GetKlineDataCommand::new(self.node_id().clone(), resp_tx, payload);

        self.send_strategy_command(get_kline_series_cmd.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
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
        let payload = CalculateHistoryIndicatorCmdPayload::new(
            self.strategy_id().clone(),
            self.node_id().clone(),
            indicator_key.get_kline_key(),
            kline_data.clone(),
            indicator_key.indicator_config.clone(),
        );
        let cmd: IndicatorEngineCommand = CalculateHistoryIndicatorCommand::new(self.node_id().clone(), resp_tx, payload).into();
        EventCenterSingleton::send_command(cmd.into()).await.unwrap();

        let response = resp_rx.await.unwrap();
        match response {
            Response::Success { payload, .. } => {
                return Ok(payload.indicators.clone());
            }
            Response::Fail { error, .. } => {
                return Err(CalculateIndicatorFailedSnafu {}.into_error(error));
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
        self.send_strategy_command(cmd.into()).await.unwrap();
        let response = resp_rx.await.unwrap();
        match response {
            StrategyResponse::Success { .. } => {
                return Ok(());
            }
            StrategyResponse::Fail { error, .. } => {
                return Ok(());
            }
        }
    }
}
