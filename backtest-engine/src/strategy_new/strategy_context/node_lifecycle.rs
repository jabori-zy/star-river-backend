use super::BacktestStrategyContext;
use strategy_core::strategy::context_trait::{StrategyIdentityExt, StrategyInfraExt, StrategyWorkflowExt};
use crate::error::{strategy_error::BacktestStrategyError};
use snafu::IntoError;
use tokio::time::Duration;
use strategy_core::{error::strategy_error::{NodeInitTimeoutSnafu, NodeStateNotReadySnafu, TokioTaskFailedSnafu}};
use crate::node_list_new::node_state_machine::NodeRunState;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use crate::error::strategy_error::UpdateStrategyStatusFailedSnafu;
use snafu::ResultExt;

#[async_trait]
impl StrategyWorkflowExt for BacktestStrategyContext {

    type Error = BacktestStrategyError;

    async fn init_node(context: Arc<RwLock<Self>>) -> Result<(), Self::Error> {
        let (strategy_name, nodes) = {
            let ctx = context.read().await;
            (ctx.strategy_name().clone(), ctx.topological_sort()?)
        }; // 读锁在这里释放

        // 逐个初始化节点，不持有锁
        for n in nodes {
            let node_clone = n.clone(); 
            let node_handle: tokio::task::JoinHandle<Result<(), BacktestStrategyError>> = tokio::spawn(async move {
                node_clone
                    .init()
                    .await?;
                Ok(())
            });

            let node_name = n.node_name().await;

            // 等待节点初始化完成（这里没有持有任何锁）
            match tokio::time::timeout(Duration::from_secs(120), node_handle).await {
                Ok(result) => {
                    if let Err(e) = result {
                        return Err(TokioTaskFailedSnafu {
                            strategy_name: strategy_name.clone(),
                            task_name: "INIT_NODE".to_string(),
                            node_name,
                        }
                        .into_error(e)
                        .into()
                    );
                }

                if let Ok(Err(e)) = result {
                    return Err(e);
                }
            }
                Err(e) => {
                    return Err(NodeInitTimeoutSnafu {
                        strategy_name: strategy_name.clone(),
                        node_name: node_name.clone(),
                    }
                    .into_error(e)
                    .into());
                }
            }

            // 等待节点进入Ready状态（这里也没有持有锁）
            let mut retry_count = 0;
            let max_retries = 10;

            while retry_count < max_retries {
                if n.is_in_state(NodeRunState::Ready).await {
                    tracing::debug!("[{node_name}] node is ready");
                    // 节点初始化间隔
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    break;
                }
                retry_count += 1;
                tokio::time::sleep(Duration::from_millis(500)).await;
            }

            if retry_count >= max_retries {
                return Err(NodeStateNotReadySnafu {
                    strategy_name: strategy_name.clone(),
                    node_name: node_name.clone(),
                }
                .fail()?);
            }
        }

        Ok(())
    }

    async fn stop_node(context: Arc<RwLock<Self>>) -> Result<(), Self::Error> {
        // 短暂持有锁获取节点列表，然后立即释放
        let (strategy_name, nodes) = {
            let ctx = context.read().await;
            (ctx.strategy_name().clone(), ctx.topological_sort()?)
        }; // 读锁在这里释放

        for n in nodes {
            let node_clone = n.clone();
            let node_handle: tokio::task::JoinHandle<Result<(), BacktestStrategyError>> = tokio::spawn(async move {
                node_clone.stop().await?;
                Ok(())
            });

            let node_name = n.node_name().await;

            // 等待节点停止完成
            match tokio::time::timeout(Duration::from_secs(10), node_handle).await {
                Ok(result) => {
                    // 处理 JoinError（任务 panic 或被取消）
                    if let Err(e) = result {
                        return Err(TokioTaskFailedSnafu {
                            strategy_name: strategy_name.clone(),
                            task_name: "STOP_NODE".to_string(),
                            node_name: node_name.clone(),
                        }
                        .into_error(e)
                        .into());
                    }

                    // 处理节点停止过程中的业务错误
                    if let Ok(Err(e)) = result {
                        return Err(e);
                    }
                }
                Err(e) => {
                    // 处理超时错误
                    return Err(NodeInitTimeoutSnafu {
                        strategy_name: strategy_name.clone(),
                        node_name: node_name.clone(),
                    }
                    .into_error(e)
                    .into());
                }
            }

            // 等待节点进入Stopped状态
            let mut retry_count = 0;
            let max_retries = 20;

            while retry_count < max_retries {
                if n.is_in_state(NodeRunState::Stopped).await {
                    tracing::debug!("[{node_name}] node is stopped");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    return Ok(());
                }
                retry_count += 1;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            if retry_count >= max_retries {
                return Err(NodeStateNotReadySnafu {
                    strategy_name: strategy_name.clone(),
                    node_name: node_name.clone(),
                }
                .fail()?);
            }
        }
        Ok(())
    }
}


impl BacktestStrategyContext {
    pub async fn wait_for_all_nodes_stopped(&self, timeout_secs: u64) -> Result<bool, BacktestStrategyError> {
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        let nodes = self.topological_sort()?;
        loop {
            let mut all_stopped = true;
            
            // 检查所有节点状态
            for node in nodes.iter() {
                if !node.is_in_state(NodeRunState::Stopped).await {
                    all_stopped = false;
                    break;
                }
            }

            // 如果所有节点都已停止，返回成功
            if all_stopped {
                tracing::info!("所有节点已停止，共耗时{}ms", start_time.elapsed().as_millis());
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


    pub async fn store_strategy_status(&mut self, status: String) -> Result<(), BacktestStrategyError> {
        let strategy_id = self.strategy_id();
        let strategy_name = self.strategy_name().clone();
        let database = self.database();

        StrategyConfigMutation::update_strategy_status(&database, strategy_id, status)
            .await
            .context(UpdateStrategyStatusFailedSnafu {
                strategy_name: strategy_name,
            })?;
        Ok(())
    } 
}
