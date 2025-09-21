use super::super::{
    NodeCommand, NodeCommandTrait, NodeResponder, StrategyCommand, StrategyCommandTrait,
    StrategyResponder,
};
use chrono::Utc;
use star_river_core::system::DateTimeUtc;
use star_river_core::custom_type::NodeId;


#[derive(Debug)]
pub enum BacktestStrategyCommand {
    GetStartNodeConfig(GetStartNodeConfigParams),
    NodeReset(NodeResetParams),
}

impl StrategyCommandTrait for BacktestStrategyCommand {
    fn node_id(&self) -> &NodeId {
        match self {
            BacktestStrategyCommand::GetStartNodeConfig(params) => &params.node_id,
            BacktestStrategyCommand::NodeReset(params) => &params.node_id,
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestStrategyCommand::GetStartNodeConfig(params) => params.datetime,
            BacktestStrategyCommand::NodeReset(params) => params.datetime,
        }
    }

    fn responder(&self) -> &StrategyResponder {
        match self {
            BacktestStrategyCommand::GetStartNodeConfig(params) => &params.responder,
            BacktestStrategyCommand::NodeReset(params) => &params.responder,
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
        StrategyCommand::BacktestStrategy(BacktestStrategyCommand::GetStartNodeConfig(params))
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
        StrategyCommand::BacktestStrategy(BacktestStrategyCommand::NodeReset(params))
    }
}

#[derive(Debug)]
pub enum BacktestNodeCommand {
    GetStrategyKeys(GetStrategyKeysParams),
    GetMinIntervalSymbols(GetMinIntervalSymbolsParams),
    GetKlineIndex(GetKlineIndexParams),
    GetCurrentTime(GetCurrentTimeParams),
}

impl NodeCommandTrait for BacktestNodeCommand {
    fn responder(&self) -> &NodeResponder {
        match self {
            BacktestNodeCommand::GetStrategyKeys(params) => &params.responder,
            BacktestNodeCommand::GetMinIntervalSymbols(params) => &params.responder,
            BacktestNodeCommand::GetKlineIndex(params) => &params.responder,
            BacktestNodeCommand::GetCurrentTime(params) => &params.responder,
        }
    }

    fn datetime(&self) -> DateTimeUtc {
        match self {
            BacktestNodeCommand::GetStrategyKeys(params) => params.datetime,
            BacktestNodeCommand::GetMinIntervalSymbols(params) => params.datetime,
            BacktestNodeCommand::GetKlineIndex(params) => params.datetime,
            BacktestNodeCommand::GetCurrentTime(params) => params.datetime,
        }
    }

    fn node_id(&self) -> &NodeId {
        match self {
            BacktestNodeCommand::GetStrategyKeys(params) => &params.node_id,
            BacktestNodeCommand::GetMinIntervalSymbols(params) => &params.node_id,
            BacktestNodeCommand::GetKlineIndex(params) => &params.node_id,
            BacktestNodeCommand::GetCurrentTime(params) => &params.node_id,
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
        NodeCommand::BacktestNode(BacktestNodeCommand::GetStrategyKeys(params))
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
        NodeCommand::BacktestNode(BacktestNodeCommand::GetMinIntervalSymbols(params))
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
        NodeCommand::BacktestNode(BacktestNodeCommand::GetCurrentTime(params))
    }
}
