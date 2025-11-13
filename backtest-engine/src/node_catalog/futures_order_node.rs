mod context;
mod event_listener;
mod futures_order_node_types;
mod node_lifecycle;
mod state_machine;

use std::sync::Arc;

use context::FuturesOrderNodeContext;
use futures::StreamExt;
use futures_order_node_types::FuturesOrderNodeBacktestConfig;
use heartbeat::Heartbeat;
use sea_orm::DatabaseConnection;
use serde_json;
use snafu::ResultExt;
use star_river_core::custom_type::{NodeId, NodeName, StrategyId};
use state_machine::{FuturesOrderNodeStateMachine, futures_order_node_transition};
use strategy_core::{
    error::node_error::{ConfigDeserializationFailedSnafu, ConfigFieldValueNullSnafu},
    node::{
        NodeBase, NodeType,
        context_trait::{NodeIdentityExt, NodeTaskControlExt},
        metadata::NodeMetadata,
        node_trait::NodeContextAccessor,
        utils::generate_strategy_output_handle,
    },
};
use tokio::sync::{Mutex, RwLock, broadcast, mpsc};
use tokio_stream::wrappers::BroadcastStream;
use virtual_trading::{command::VtsCommand, event::VtsEvent};

use crate::{
    node::{
        node_command::BacktestNodeCommand, node_error::BacktestNodeError, node_event::BacktestNodeEvent, node_state_machine::NodeRunState,
    },
    strategy::{PlayIndex, strategy_command::BacktestStrategyCommand},
};

#[derive(Debug, Clone)]
pub struct FuturesOrderNode {
    inner: NodeBase<FuturesOrderNodeContext>,
}

impl std::ops::Deref for FuturesOrderNode {
    type Target = NodeBase<FuturesOrderNodeContext>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl NodeContextAccessor for FuturesOrderNode {
    type Context = FuturesOrderNodeContext;
    fn context(&self) -> &Arc<RwLock<Self::Context>> {
        self.inner.context()
    }
}

impl FuturesOrderNode {
    pub fn new(
        node_config: serde_json::Value,
        strategy_command_sender: mpsc::Sender<BacktestStrategyCommand>,
        node_command_receiver: Arc<Mutex<mpsc::Receiver<BacktestNodeCommand>>>,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        vts_command_sender: mpsc::Sender<VtsCommand>,
        vts_event_receiver: broadcast::Receiver<VtsEvent>,
    ) -> Result<Self, BacktestNodeError> {
        let (strategy_id, node_id, node_name, node_config) = Self::check_futures_order_node_config(node_config)?;
        let strategy_bound_handle = generate_strategy_output_handle::<BacktestNodeEvent>(&node_id);
        let state_machine = FuturesOrderNodeStateMachine::new(node_name.clone(), NodeRunState::Created, futures_order_node_transition);

        let metadata = NodeMetadata::new(
            strategy_id,
            node_id,
            node_name,
            NodeType::FuturesOrderNode,
            state_machine,
            strategy_bound_handle,
            strategy_command_sender,
            node_command_receiver,
        );
        let context = FuturesOrderNodeContext::new(
            metadata,
            node_config,
            play_index_watch_rx,
            database,
            heartbeat,
            vts_command_sender,
            vts_event_receiver,
        );
        Ok(Self {
            inner: NodeBase::new(context),
        })
    }

    fn check_futures_order_node_config(
        node_config: serde_json::Value,
    ) -> Result<(StrategyId, NodeId, NodeName, FuturesOrderNodeBacktestConfig), BacktestNodeError> {
        let node_id = node_config
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "id".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_data = node_config
            .get("data")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "data".to_string(),
                }
                .build()
            })?
            .to_owned();
        let node_name = node_data
            .get("nodeName")
            .and_then(|name| name.as_str())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "nodeName".to_string(),
                }
                .build()
            })?
            .to_owned();
        let strategy_id = node_data
            .get("strategyId")
            .and_then(|id| id.as_i64())
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "strategyId".to_string(),
                }
                .build()
            })?
            .to_owned() as StrategyId;

        let backtest_config_json = node_data
            .get("backtestConfig")
            .ok_or_else(|| {
                ConfigFieldValueNullSnafu {
                    field_name: "backtestConfig".to_string(),
                }
                .build()
            })?
            .to_owned();

        let node_config =
            serde_json::from_value::<FuturesOrderNodeBacktestConfig>(backtest_config_json).context(ConfigDeserializationFailedSnafu {})?;
        Ok((strategy_id, node_id, node_name, node_config))
    }

    async fn listen_vts_events(&self) {
        let (vts_event_receiver, cancel_token, node_name) = self
            .with_ctx_read(|ctx| {
                let receiver = ctx.vts_event_receiver.resubscribe();
                let cancel_token = ctx.cancel_token().clone();
                let node_name = ctx.node_name().clone();
                (receiver, cancel_token, node_name)
            })
            .await;

        // Create a stream for receiving VTS events
        let mut stream = BroadcastStream::new(vts_event_receiver);
        let context = self.context().clone();

        // Spawn task to receive VTS events
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // If cancel signal is triggered, abort task
                    _ = cancel_token.cancelled() => {
                        tracing::info!("[{}] virtual trading system events listener stopped", node_name);
                        break;
                    }
                    // Receive events
                    receive_result = stream.next() => {
                        match receive_result {
                            Some(Ok(event)) => {
                                let mut context_guard = context.write().await;
                                if let Err(e) = context_guard.handle_vts_event(event).await {
                                    tracing::error!("[{}] failed to handle virtual trading system event: {}", node_name, e);
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("[{}] failed to receive VTS event: {}", node_name, e);
                            }
                            None => {
                                tracing::warn!("[{}] VTS event stream closed", node_name);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}
