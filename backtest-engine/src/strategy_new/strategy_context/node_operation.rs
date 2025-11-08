use snafu::IntoError;
// third-party
use tokio::sync::oneshot;


// current crate
use super::BacktestStrategyContext;
use crate::{error::strategy_error::{
    BacktestStrategyError, GetStartNodeConfigFailedSnafu
}, node_command::{GetStartNodeConfigCmdPayload, GetStartNodeConfigCommand}};
use strategy_core::{
    communication::node::NodeResponse, 
    strategy::{
        context_trait::{StrategyCommunicationExt, StrategyIdentityExt}
    }
};
use crate::strategy_new::config::BacktestStrategyConfig;

impl BacktestStrategyContext {
    // 获取start节点配置
    pub async fn get_start_node_config(&self) -> Result<BacktestStrategyConfig, BacktestStrategyError> {
        let (resp_tx, resp_rx) = oneshot::channel();

        let payload = GetStartNodeConfigCmdPayload{};
        let cmd = GetStartNodeConfigCommand::new("start_node".to_string(), resp_tx, payload);

        self.send_node_command(cmd.into()).await;

        let response = resp_rx.await.unwrap();
        match response {
            NodeResponse::Success { payload, .. } => Ok(payload.backtest_strategy_config.clone()),
            NodeResponse::Fail { error, .. } => {
                let e = GetStartNodeConfigFailedSnafu {
                    strategy_name: self.strategy_name().clone(),
                }.into_error(error);
                Err(e)
            }
        }
    }

    
}
