use super::{
    BacktestStrategyContext,
    BacktestNodeRunState,
    BacktestNodeTrait,
};
use star_river_core::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use snafu::{ResultExt, IntoError};
use tokio::time::Duration;


impl BacktestStrategyContext {
        // 初始化所有节点的方法，不持有外部锁
    pub async fn init_node(
        context: Arc<RwLock<Self>>,
    ) -> Result<(), BacktestStrategyError> {
        // 短暂持有锁获取节点列表
        let nodes = {
            let context_guard = context.read().await;
            context_guard.topological_sort()
        }; // 锁立即释放

        // 逐个初始化节点，不持有锁
        for node in nodes {
            let mut node_clone = node.clone();

            let node_handle: tokio::task::JoinHandle<Result<(), BacktestStrategyError>> =
                tokio::spawn(async move {
                    node_clone.init().await.context(NodeInitSnafu {})?;
                    Ok(())
                });

            let node_name = node.get_node_name().await;
            let node_id = node.get_node_id().await;
            let node_type = node.get_node_type().await;

            // 等待节点初始化完成（这里没有持有任何锁）
            match tokio::time::timeout(Duration::from_secs(30), node_handle).await {
                Ok(result) => {
                    if let Err(e) = result {
                        return Err(TokioTaskFailedSnafu {
                            task_name: "INIT_NODE".to_string(),
                            node_name,
                            node_id,
                            node_type: node_type.to_string(),
                        }
                        .into_error(e));
                    }

                    if let Ok(Err(e)) = result {
                        return Err(e);
                    }
                }
                Err(e) => {
                    return Err(NodeInitTimeoutSnafu {
                        node_id,
                        node_name,
                        node_type: node_type.to_string(),
                    }
                    .into_error(e));
                }
            }

            // 等待节点进入Ready状态（这里也没有持有锁）
            let mut retry_count = 0;
            let max_retries = 10;

            while retry_count < max_retries {
                let run_state = node.get_run_state().await;
                if run_state == BacktestNodeRunState::Ready {
                    tracing::debug!("节点 {} 已进入Ready状态", node_id);
                    // 节点初始化间隔
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    break;
                }
                retry_count += 1;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }

            if retry_count >= max_retries {
                return Err(NodeStateNotReadySnafu {
                    node_id: node_id,
                    node_name: node_name,
                    node_type: node_type.to_string(),
                }
                .fail()?);
            }
        }

        Ok(())
    }


    pub async fn stop_node(&self, node: Box<dyn BacktestNodeTrait>) -> Result<(), String> {
        let mut node_clone = node.clone();
        let node_name = node_clone.get_node_name().await;
        let node_id = node_clone.get_node_id().await;

        let node_handle = tokio::spawn(async move {
            if let Err(e) = node_clone.stop().await {
                tracing::error!(node_name = %node_name, node_id = %node_id, error = %e, "节点停止失败。");
                return Err(format!("节点停止失败。"));
            }
            Ok(())
        });

        let node_name = node.get_node_name().await;
        let node_id = node.get_node_id().await;

        // 等待节点停止完成
        match tokio::time::timeout(Duration::from_secs(10), node_handle).await {
            Ok(result) => {
                if let Err(e) = result {
                    return Err(format!("节点 {} 停止任务失败: {}", node_name, e));
                }

                if let Ok(Err(e)) = result {
                    return Err(format!("节点 {} 停止过程中出错: {}", node_name, e));
                }
            }
            Err(_) => {
                return Err(format!("节点 {} 停止超时", node_id));
            }
        }

        // 等待节点进入Stopped状态
        let mut retry_count = 0;
        let max_retries = 20;

        while retry_count < max_retries {
            let run_state = node.get_run_state().await;
            if run_state == BacktestNodeRunState::Stopped {
                tracing::debug!("节点 {} 已进入Stopped状态", node_id);
                tokio::time::sleep(Duration::from_millis(10)).await;
                return Ok(());
            }
            retry_count += 1;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Err(format!("节点 {} 未能进入Stopped状态", node_id))
    }


    pub async fn wait_for_all_nodes_stopped(&self, timeout_secs: u64) -> Result<bool, String> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            let mut all_stopped = true;
            // 检查所有节点状态
            for node in self.graph.node_weights() {
                let run_state = node.get_run_state().await;
                if run_state != BacktestNodeRunState::Stopped {
                    all_stopped = false;
                    break;
                }
            }

            // 如果所有节点都已停止，返回成功
            if all_stopped {
                tracing::info!(
                    "所有节点已停止，共耗时{}ms",
                    start_time.elapsed().as_millis()
                );
                return Ok(true);
            }

            // 检查是否超时
            if start_time.elapsed() > timeout {
                tracing::warn!("等待节点停止超时，已等待{}秒", timeout_secs);
                return Ok(false);
            }

            // 短暂休眠后再次检查
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }
    }

}