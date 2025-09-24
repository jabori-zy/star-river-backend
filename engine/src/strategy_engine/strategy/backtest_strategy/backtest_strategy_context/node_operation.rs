use super::{
    BacktestNodeTrait, BacktestStrategyConfig, BacktestStrategyContext, BacktestStrategyError,
    BacktestNodeResponse, GetStartNodeConfigFailedSnafu, GetStartNodeConfigParams, StrategyResponse,
};
use tokio::sync::oneshot;

impl BacktestStrategyContext {
    // 拓扑排序
    pub fn topological_sort(&self) -> Vec<Box<dyn BacktestNodeTrait>> {
        petgraph::algo::toposort(&self.graph, None)
            .unwrap_or_default()
            .into_iter()
            .map(|index| self.graph[index].clone())
            .collect()
    }

    // 获取start节点配置
    pub async fn get_start_node_config(&self) -> Result<BacktestStrategyConfig, BacktestStrategyError> {
        let (resp_tx, resp_rx) = oneshot::channel();
        let get_start_node_config_params = GetStartNodeConfigParams::new("start_node".to_string(), resp_tx);
        self.strategy_command_publisher
            .send(get_start_node_config_params.into())
            .await
            .unwrap();
        // EventCenterSingleton::send_command(get_start_node_config_command).await.unwrap();
        let response = resp_rx.await.unwrap();
        if response.success() {
            if let StrategyResponse::BacktestStrategy(BacktestNodeResponse::GetStartNodeConfig(
                get_start_node_config_response,
            )) = response
            {
                Ok(get_start_node_config_response.backtest_strategy_config)
            } else {
                Err(GetStartNodeConfigFailedSnafu {
                    strategy_name: self.strategy_name.clone(),
                }
                .fail()?)
            }
        } else {
            Err(GetStartNodeConfigFailedSnafu {
                strategy_name: self.strategy_name.clone(),
            }
            .fail()?)
        }
    }
}
