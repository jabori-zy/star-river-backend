use snafu::IntoError;
use star_river_core::custom_type::{NodeId, NodeName};
use strategy_core::{
    communication::node::NodeResponse,
    strategy::context_trait::{StrategyCommunicationExt, StrategyIdentityExt},
};
// third-party
use tokio::sync::oneshot;

// current crate
use super::BacktestStrategyContext;
use crate::{
    node::node_command::{
        GetFuturesOrderConfigCmdPayload, GetFuturesOrderConfigCommand, GetStartNodeConfigCmdPayload, GetStartNodeConfigCommand,
    },
    node_catalog::futures_order_node::FuturesOrderNodeConfig,
    strategy::{
        strategy_config::BacktestStrategyConfig,
        strategy_error::{BacktestStrategyError, GetNodeConfigFailedSnafu, GetStartNodeConfigFailedSnafu},
    },
};

impl BacktestStrategyContext {
    // 获取start节点配置
    pub async fn get_strategy_config(&self) -> Result<BacktestStrategyConfig, BacktestStrategyError> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let payload = GetStartNodeConfigCmdPayload {};
        let cmd = GetStartNodeConfigCommand::new("start_node".to_string(), "start_node".to_string(), resp_tx, payload);

        self.send_node_command(cmd.into()).await.unwrap();

        let response = resp_rx.await.unwrap();
        match response {
            NodeResponse::Success { payload, .. } => Ok(payload.backtest_strategy_config.clone()),
            NodeResponse::Fail { error, .. } => {
                let e = GetStartNodeConfigFailedSnafu {
                    strategy_name: self.strategy_name().clone(),
                }
                .into_error(error);
                Err(e)
            }
        }
    }

    pub async fn get_futures_order_config(
        &self,
        node_id: &NodeId,
        node_name: &NodeName,
    ) -> Result<FuturesOrderNodeConfig, BacktestStrategyError> {
        let (tx, rx) = oneshot::channel();
        let payload = GetFuturesOrderConfigCmdPayload {};
        let cmd = GetFuturesOrderConfigCommand::new(node_id.clone(), node_name.clone(), tx, payload);
        tracing::debug!("send get futures order config command to node @[{node_name}]");
        self.send_node_command(cmd.into()).await.unwrap();
        tracing::debug!("received get futures order config response from node @[{node_name}]");
        match rx.await.unwrap() {
            NodeResponse::Success { payload, .. } => Ok(payload.futures_order_node_config.clone()),
            NodeResponse::Fail { error, .. } => {
                let e = GetNodeConfigFailedSnafu {
                    strategy_name: self.strategy_name().clone(),
                    node_name: node_name.clone(),
                }
                .into_error(error);
                Err(e)
            }
        }
    }
}
