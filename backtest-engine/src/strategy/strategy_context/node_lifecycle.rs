use std::sync::Arc;

use async_trait::async_trait;
use database::mutation::strategy_config_mutation::StrategyConfigMutation;
use snafu::{IntoError, ResultExt};
use strategy_core::{
    error::strategy_error::{NodeInitTimeoutSnafu, NodeStateNotReadySnafu, TokioTaskFailedSnafu},
    strategy::context_trait::{StrategyIdentityExt, StrategyInfraExt, StrategyWorkflowExt},
};
use tokio::{sync::RwLock, time::Duration};

use super::BacktestStrategyContext;
use crate::{
    node::node_state_machine::NodeRunState,
    strategy::strategy_error::{BacktestStrategyError, UpdateStrategyStatusFailedSnafu},
};

const INIT_NODE_INTERVAL: u64 = 0;

#[async_trait]
impl StrategyWorkflowExt for BacktestStrategyContext {
    async fn init_node(context: Arc<RwLock<Self>>) -> Result<(), Self::Error> {
        let (strategy_name, nodes) = {
            let ctx = context.read().await;
            (ctx.strategy_name().clone(), ctx.topological_sort()?)
        }; // Read lock is released here

        // Initialize nodes one by one without holding lock
        for n in nodes {
            let node_clone = n.clone();
            let node_handle: tokio::task::JoinHandle<Result<(), BacktestStrategyError>> = tokio::spawn(async move {
                node_clone.init().await?;
                Ok(())
            });

            let node_name = n.node_name().await;

            // Wait for node initialization to complete (no lock held here)
            match tokio::time::timeout(Duration::from_secs(120), node_handle).await {
                Ok(result) => {
                    if let Err(e) = result {
                        return Err(TokioTaskFailedSnafu {
                            strategy_name: strategy_name.clone(),
                            task_name: "INIT_NODE".to_string(),
                            node_name,
                        }
                        .into_error(e)
                        .into());
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

            // Wait for node to enter Ready state (no lock held here either)
            let mut retry_count = 0;
            let max_retries = 10;

            while retry_count < max_retries {
                if n.is_in_state(NodeRunState::Ready).await {
                    tracing::debug!("[{node_name}] node is ready");
                    // Node initialization interval
                    tokio::time::sleep(Duration::from_millis(INIT_NODE_INTERVAL)).await;
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
        // Briefly hold lock to get node list, then immediately release
        let (strategy_name, nodes) = {
            let ctx = context.read().await;
            (ctx.strategy_name().clone(), ctx.topological_sort()?)
        }; // Read lock is released here

        for n in nodes {
            let node_clone = n.clone();
            let node_handle: tokio::task::JoinHandle<Result<(), BacktestStrategyError>> = tokio::spawn(async move {
                node_clone.stop().await?;
                Ok(())
            });

            let node_name = n.node_name().await;
            tracing::debug!("@[{node_name}] stopping...");

            // Wait for node stop to complete
            match tokio::time::timeout(Duration::from_secs(10), node_handle).await {
                Ok(result) => {
                    // Handle JoinError (task panic or cancelled)
                    if let Err(e) = result {
                        return Err(TokioTaskFailedSnafu {
                            strategy_name: strategy_name.clone(),
                            task_name: "STOP_NODE".to_string(),
                            node_name: node_name.clone(),
                        }
                        .into_error(e)
                        .into());
                    }

                    // Handle business errors during node stop
                    if let Ok(Err(e)) = result {
                        return Err(e);
                    }
                }
                Err(e) => {
                    // Handle timeout error
                    return Err(NodeInitTimeoutSnafu {
                        strategy_name: strategy_name.clone(),
                        node_name: node_name.clone(),
                    }
                    .into_error(e)
                    .into());
                }
            }

            // Wait for node to enter Stopped state
            let mut retry_count = 0;
            let max_retries = 20;

            while retry_count < max_retries {
                if n.is_in_state(NodeRunState::Stopped).await {
                    tracing::debug!("@[{node_name}] node is stopped");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                    break;
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

            // Check all node states
            for node in nodes.iter() {
                if !node.is_in_state(NodeRunState::Stopped).await {
                    all_stopped = false;
                    break;
                }
            }

            // If all nodes stopped, return success
            if all_stopped {
                tracing::info!("All nodes stopped, took {}ms", start_time.elapsed().as_millis());
                return Ok(true);
            }

            // Check if timeout
            if start_time.elapsed() > timeout {
                tracing::warn!("Wait for nodes stop timeout, waited {} seconds", timeout_secs);
                return Ok(false);
            }

            // Brief sleep then check again
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
