// third-party
use tokio::sync::oneshot;


// current crate
use super::{BacktestNode, BacktestStrategyConfig, BacktestStrategyContext};
use crate::error::strategy_error::{
    BacktestStrategyError, GetStartNodeConfigFailedSnafu, NodeCycleDetectedSnafu,
};

impl BacktestStrategyContext {
    // 拓扑排序
    pub fn topological_sort(&self) -> Result<Vec<BacktestNode>, BacktestStrategyError> {
        let result = petgraph::algo::toposort(&self.graph, None);
        match result {
            Ok(nodes_index) => Ok(nodes_index.into_iter().map(|index| self.graph[index].clone()).collect()),
            Err(_) => {
                let error = NodeCycleDetectedSnafu {
                    strategy_name: self.strategy_name().clone(),
                }.build();
                return Err(error);
            }
        }
    }

    // 获取start节点配置
    pub async fn get_start_node_config(&self) -> Result<BacktestStrategyConfig, BacktestStrategyError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = GetStartNodeConfigCommand::new("start_node".to_string(), resp_tx, None);

        self.send_node_command(cmd.into()).await;

        let response = resp_rx.await.unwrap();
        if response.is_success() {
            Ok(response.backtest_strategy_config.clone())
        } else {
            Err(GetStartNodeConfigFailedSnafu {
                strategy_name: self.strategy_name.clone(),
            }
            .fail()?)
        }
    }
}
