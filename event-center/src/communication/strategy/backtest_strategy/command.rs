use super::super::{
    NodeCommand, NodeCommandTrait, NodeResponder, StrategyCommand, StrategyCommandTrait, StrategyResponder,
};
use chrono::Utc;
use star_river_core::cache::key::KlineKey;
use star_river_core::custom_type::{NodeId, PlayIndex};
use star_river_core::indicator::Indicator;
use star_river_core::market::Kline;
use star_river_core::system::DateTimeUtc;

#[derive(Debug)]
pub enum BacktestNodeCommand {
    GetStartNodeConfig(GetStartNodeConfigParams),
    NodeReset(NodeResetParams),
}

impl StrategyCommandTrait for BacktestNodeCommand {
    fn node_id(&self) -> &NodeId {
        match self {
            BacktestNodeCommand::GetStartNodeConfig(params) => &params.node_id,
            BacktestNodeCommand::NodeReset(params) => &params.node_id,
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestNodeCommand::GetStartNodeConfig(params) => params.datetime,
            BacktestNodeCommand::NodeReset(params) => params.datetime,
        }
    }

    fn responder(&self) -> &StrategyResponder {
        match self {
            BacktestNodeCommand::GetStartNodeConfig(params) => &params.responder,
            BacktestNodeCommand::NodeReset(params) => &params.responder,
        }
    }
}

#[derive(Debug)]
pub struct GetStartNodeConfigParams {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: StrategyResponder,
}

impl GetStartNodeConfigParams {
    pub fn new(node_id: NodeId, responder: StrategyResponder) -> Self {
        Self {
            node_id,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetStartNodeConfigParams> for StrategyCommand {
    fn from(params: GetStartNodeConfigParams) -> Self {
        StrategyCommand::BacktestStrategy(BacktestNodeCommand::GetStartNodeConfig(params))
    }
}

#[derive(Debug)]
pub struct NodeResetParams {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: StrategyResponder,
}

impl NodeResetParams {
    pub fn new(node_id: NodeId, responder: StrategyResponder) -> Self {
        Self {
            node_id,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<NodeResetParams> for StrategyCommand {
    fn from(params: NodeResetParams) -> Self {
        StrategyCommand::BacktestStrategy(BacktestNodeCommand::NodeReset(params))
    }
}

#[derive(Debug)]
pub enum BacktestStrategyCommand {
    GetStrategyKeys(GetStrategyKeysParams),
    GetMinIntervalSymbols(GetMinIntervalSymbolsParams),
    GetKlineIndex(GetKlineIndexParams),
    GetCurrentTime(GetCurrentTimeParams),
    InitKlineData(InitKlineDataParams),
    InitIndicatorData(InitIndicatorDataParams),
    GetKlineData(GetKlineDataParams),
    GetIndicatorData(GetIndicatorDataParams),
    UpdateKlineData(UpdateKlineDataParams),
    UpdateIndicatorData(UpdateIndicatorDataParams),
}

impl NodeCommandTrait for BacktestStrategyCommand {
    fn responder(&self) -> &NodeResponder {
        match self {
            BacktestStrategyCommand::GetStrategyKeys(params) => &params.responder,
            BacktestStrategyCommand::GetMinIntervalSymbols(params) => &params.responder,
            BacktestStrategyCommand::GetKlineIndex(params) => &params.responder,
            BacktestStrategyCommand::GetCurrentTime(params) => &params.responder,
            BacktestStrategyCommand::InitKlineData(params) => &params.responder,
            BacktestStrategyCommand::InitIndicatorData(params) => &params.responder,
            BacktestStrategyCommand::GetKlineData(params) => &params.responder,
            BacktestStrategyCommand::GetIndicatorData(params) => &params.responder,
            BacktestStrategyCommand::UpdateKlineData(params) => &params.responder,
            BacktestStrategyCommand::UpdateIndicatorData(params) => &params.responder,
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestStrategyCommand::GetStrategyKeys(params) => params.datetime,
            BacktestStrategyCommand::GetMinIntervalSymbols(params) => params.datetime,
            BacktestStrategyCommand::GetKlineIndex(params) => params.datetime,
            BacktestStrategyCommand::GetCurrentTime(params) => params.datetime,
            BacktestStrategyCommand::InitKlineData(params) => params.datetime,
            BacktestStrategyCommand::InitIndicatorData(params) => params.datetime,
            BacktestStrategyCommand::GetKlineData(params) => params.datetime,
            BacktestStrategyCommand::GetIndicatorData(params) => params.datetime,
            BacktestStrategyCommand::UpdateKlineData(params) => params.datetime,
            BacktestStrategyCommand::UpdateIndicatorData(params) => params.datetime,
        }
    }

    fn node_id(&self) -> &NodeId {
        match self {
            BacktestStrategyCommand::GetStrategyKeys(params) => &params.node_id,
            BacktestStrategyCommand::GetMinIntervalSymbols(params) => &params.node_id,
            BacktestStrategyCommand::GetKlineIndex(params) => &params.node_id,
            BacktestStrategyCommand::GetCurrentTime(params) => &params.node_id,
            BacktestStrategyCommand::InitKlineData(params) => &params.node_id,
            BacktestStrategyCommand::InitIndicatorData(params) => &params.node_id,
            BacktestStrategyCommand::GetKlineData(params) => &params.node_id,
            BacktestStrategyCommand::GetIndicatorData(params) => &params.node_id,
            BacktestStrategyCommand::UpdateKlineData(params) => &params.node_id,
            BacktestStrategyCommand::UpdateIndicatorData(params) => &params.node_id,
        }
    }
}

#[derive(Debug)]
pub struct GetStrategyKeysParams {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl GetStrategyKeysParams {
    pub fn new(node_id: NodeId, responder: NodeResponder) -> Self {
        Self {
            node_id,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetStrategyKeysParams> for NodeCommand {
    fn from(params: GetStrategyKeysParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::GetStrategyKeys(params))
    }
}

#[derive(Debug)]
pub struct GetKlineIndexParams {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl GetKlineIndexParams {
    pub fn new(node_id: NodeId, responder: NodeResponder) -> Self {
        Self {
            node_id,
            datetime: Utc::now(),
            responder,
        }
    }
}

#[derive(Debug)]
pub struct GetMinIntervalSymbolsParams {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl GetMinIntervalSymbolsParams {
    pub fn new(node_id: NodeId, responder: NodeResponder) -> Self {
        Self {
            node_id,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetMinIntervalSymbolsParams> for NodeCommand {
    fn from(params: GetMinIntervalSymbolsParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::GetMinIntervalSymbols(params))
    }
}

#[derive(Debug)]
pub struct GetCurrentTimeParams {
    pub node_id: NodeId,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl GetCurrentTimeParams {
    pub fn new(node_id: NodeId, responder: NodeResponder) -> Self {
        Self {
            node_id,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetCurrentTimeParams> for NodeCommand {
    fn from(params: GetCurrentTimeParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::GetCurrentTime(params))
    }
}

#[derive(Debug)]
pub struct InitKlineDataParams {
    pub node_id: NodeId,
    pub kline_key: KlineKey,
    pub init_kline_data: Vec<Kline>,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl InitKlineDataParams {
    pub fn new(node_id: NodeId, kline_key: KlineKey, init_kline_data: Vec<Kline>, responder: NodeResponder) -> Self {
        Self {
            node_id,
            kline_key,
            init_kline_data,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<InitKlineDataParams> for NodeCommand {
    fn from(params: InitKlineDataParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::InitKlineData(params))
    }
}

#[derive(Debug)]
pub struct InitIndicatorDataParams {
    pub node_id: NodeId,
    pub indicator_key: String,
    pub init_indicator_data: Vec<Indicator>,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl InitIndicatorDataParams {
    pub fn new(
        node_id: NodeId,
        indicator_key: String,
        init_indicator_data: Vec<Indicator>,
        responder: NodeResponder,
    ) -> Self {
        Self {
            node_id,
            indicator_key,
            init_indicator_data,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<InitIndicatorDataParams> for NodeCommand {
    fn from(params: InitIndicatorDataParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::InitIndicatorData(params))
    }
}

#[derive(Debug)]
pub struct GetKlineDataParams {
    pub node_id: NodeId,
    pub kline_key: KlineKey,
    pub play_index: Option<PlayIndex>,
    pub limit: Option<i32>,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl GetKlineDataParams {
    pub fn new(node_id: NodeId, kline_key: KlineKey, play_index: Option<PlayIndex>, limit: Option<i32>, responder: NodeResponder) -> Self {
        Self {
            node_id,
            kline_key,
            play_index,
            limit,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<GetKlineDataParams> for NodeCommand {
    fn from(params: GetKlineDataParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::GetKlineData(params))
    }
}

#[derive(Debug)]
pub struct GetIndicatorDataParams {
    pub node_id: NodeId,
    pub indicator_key: String,
    pub play_index: PlayIndex,
    pub limit: Option<i32>,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl GetIndicatorDataParams {
    pub fn new(node_id: NodeId, indicator_key: String, play_index: PlayIndex, limit: Option<i32>, responder: NodeResponder) -> Self {
        Self {
            node_id,
            indicator_key,
            play_index,
            limit,
            datetime: Utc::now(),
            responder,
        }
    }
}

#[derive(Debug)]
pub struct UpdateKlineDataParams {
    pub node_id: NodeId,
    pub kline_key: KlineKey,
    pub kline: Kline,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl UpdateKlineDataParams {
    pub fn new(node_id: NodeId, kline_key: KlineKey, update_kline_data: Kline, responder: NodeResponder) -> Self {
        Self {
            node_id,
            kline_key,
            kline: update_kline_data,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<UpdateKlineDataParams> for NodeCommand {
    fn from(params: UpdateKlineDataParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::UpdateKlineData(params))
    }
}

#[derive(Debug)]
pub struct UpdateIndicatorDataParams {
    pub node_id: NodeId,
    pub indicator_key: String,
    pub update_indicator_data: Indicator,
    pub datetime: DateTimeUtc,
    pub responder: NodeResponder,
}

impl UpdateIndicatorDataParams {
    pub fn new(node_id: NodeId, indicator_key: String, update_indicator_data: Indicator, responder: NodeResponder) -> Self {
        Self {
            node_id,
            indicator_key,
            update_indicator_data,
            datetime: Utc::now(),
            responder,
        }
    }
}

impl From<UpdateIndicatorDataParams> for NodeCommand {
    fn from(params: UpdateIndicatorDataParams) -> Self {
        NodeCommand::BacktestNode(BacktestStrategyCommand::UpdateIndicatorData(params))
    }
}