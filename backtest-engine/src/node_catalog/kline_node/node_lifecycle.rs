use async_trait::async_trait;
use snafu::{IntoError, Report};
use strategy_core::{
    NodeType,
    event::{log_event::NodeStateLogEvent, node_common_event::CommonEvent},
    node::{
        context_trait::{NodeHandleExt, NodeInfoExt, NodeStateMachineExt, NodeTaskControlExt},
        node_state_machine::StateMachine,
        node_trait::{NodeContextAccessor, NodeEventListener, NodeLifecycle},
    },
};

use super::{KlineNode, state_machine::KlineNodeAction};
use crate::node::{
    node_error::{BacktestNodeError, kline_node_error::RegisterExchangeFailedSnafu},
    node_message::{common_log_message::*, kline_node_log_message::*},
    node_state_machine::{NodeRunState, NodeStateTransTrigger},
    node_utils::NodeUtils,
};

#[async_trait]
impl NodeLifecycle for KlineNode {
    type Error = BacktestNodeError;

    type Trigger = NodeStateTransTrigger;

    async fn init(&self) -> Result<(), Self::Error> {
        NodeUtils::init_node(self, None).await
    }

    async fn stop(&self) -> Result<(), BacktestNodeError> {
        NodeUtils::stop_node(self, None).await
    }

    async fn update_node_state(&self, trans_trigger: Self::Trigger) -> Result<(), Self::Error> {
        let (node_name, node_id, strategy_id, strategy_output_handle, mut state_machine) = self
            .with_ctx_read(|ctx| {
                let node_name = ctx.node_name().to_string();
                let node_id = ctx.node_id().clone();
                let strategy_id = ctx.strategy_id().clone();
                let strategy_output_handle = ctx.strategy_bound_handle().clone();
                let state_machine = ctx.state_machine().clone();
                (node_name, node_id, strategy_id, strategy_output_handle, state_machine)
            })
            .await;

        let transition_result = {
            let mut state_machine = state_machine.write().await;
            state_machine.transition(trans_trigger)?
        };

        // 执行转换后需要执行的动作
        for action in transition_result.actions() {
            // 克隆actions避免移动问题
            let (previous_state, current_state) = {
                let state_machine = state_machine.read().await;
                (state_machine.previous_state().clone(), state_machine.current_state().clone())
            };

            match action {
                KlineNodeAction::LogTransition => {
                    tracing::debug!("[{node_name}] state transition: {:?} -> {:?}", previous_state, current_state);
                }
                KlineNodeAction::LogNodeState => {
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_info_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::KlineNode,
                        log_message.to_string(),
                        current_state,
                        KlineNodeAction::LogNodeState,
                        &strategy_output_handle,
                    )
                    .await;
                }
                KlineNodeAction::ListenAndHandleExternalEvents => {
                    tracing::info!("[{node_name}] starting to listen external events");
                    let log_message = ListenExternalEventsMsg::new(node_name.clone());
                    NodeUtils::send_info_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::KlineNode,
                        log_message.to_string(),
                        current_state,
                        KlineNodeAction::ListenAndHandleExternalEvents,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_engine_event().await;
                }
                KlineNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] starting to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_info_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::KlineNode,
                        log_message.to_string(),
                        current_state,
                        KlineNodeAction::ListenAndHandleNodeEvents,
                        &strategy_output_handle,
                    )
                    .await;

                    self.listen_source_node_events().await;
                }
                KlineNodeAction::GetMinIntervalSymbols => {
                    tracing::info!("[{node_name}] start to get min interval symbols");
                    let result = self
                        .with_ctx_write_async(|ctx| {
                            Box::pin(async move {
                                ctx.get_min_interval_symbols().await.map(|min_interval_symbols| {
                                    ctx.set_min_interval_symbols(min_interval_symbols);
                                })
                            })
                        })
                        .await;

                    match result {
                        Ok(()) => {
                            let log_message = GetMinIntervalSymbolsSuccessMsg::new(node_name.clone());
                            NodeUtils::send_info_status_event(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::KlineNode,
                                log_message.to_string(),
                                current_state,
                                KlineNodeAction::GetMinIntervalSymbols,
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(err) => {
                            tracing::warn!("[{node_name}] failed to get min interval symbols from strategy: {err}");
                        }
                    }
                }
                KlineNodeAction::RegisterExchange => {
                    tracing::info!("[{node_name}] start to register exchange");

                    let (exchange, response) = self
                        .with_ctx_write_async(|ctx| {
                            Box::pin(async move {
                                // 1. 获取交易所信息
                                let exchange = ctx
                                    .node_config
                                    .exchange_mode_config
                                    .as_ref()
                                    .unwrap()
                                    .selected_account
                                    .exchange
                                    .clone();

                                // 2. register exchange
                                let response = ctx.register_exchange().await.unwrap();

                                // 返回结果供外部处理
                                (exchange, response)
                            })
                        })
                        .await;

                    // 发送开始注册日志
                    let log_message = StartRegisterExchangeMsg::new(node_name.clone(), exchange.clone());
                    NodeUtils::send_info_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        NodeType::KlineNode,
                        log_message.to_string(),
                        current_state.clone(),
                        KlineNodeAction::RegisterExchange,
                        &strategy_output_handle,
                    )
                    .await;

                    if response.is_success() {
                        let log_message = RegisterExchangeSuccessMsg::new(node_name.clone(), exchange);
                        NodeUtils::send_info_status_event(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            NodeType::KlineNode,
                            log_message.to_string(),
                            current_state,
                            KlineNodeAction::RegisterExchange,
                            &strategy_output_handle,
                        )
                        .await;
                    } else {
                        // 转换状态 Failed
                        let error = response.error().unwrap();
                        let kline_error = RegisterExchangeFailedSnafu {
                            node_name: node_name.clone(),
                        }
                        .into_error(error.clone());

                        NodeUtils::send_error_status_event(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            NodeType::KlineNode,
                            KlineNodeAction::RegisterExchange,
                            &kline_error,
                            &strategy_output_handle,
                        )
                        .await;
                        return Err(kline_error.into());
                    }
                }
                KlineNodeAction::LoadHistoryFromExchange => {
                    tracing::info!("[{node_name}] starting to load kline data from exchange");
                    let load_result = self
                        .with_ctx_write_async(|ctx| Box::pin(async move { ctx.load_kline_history_from_exchange().await }))
                        .await;

                    match load_result {
                        Ok(()) => {
                            tracing::info!("[{node_name}] load kline history from exchange success");
                            let log_message = LoadKlineDataSuccessMsg::new(node_name.clone());
                            NodeUtils::send_info_status_event(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::KlineNode,
                                log_message.to_string(),
                                current_state,
                                KlineNodeAction::LoadHistoryFromExchange,
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(err) => {
                            let report = snafu::Report::from_error(&err);
                            tracing::error!("{}", report);
                            NodeUtils::send_error_status_event(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                NodeType::KlineNode,
                                KlineNodeAction::LoadHistoryFromExchange,
                                &err,
                                &strategy_output_handle,
                            )
                            .await;
                            return Err(err.into());
                        }
                    }
                }
                KlineNodeAction::ListenAndHandleStrategyCommand => {
                    tracing::info!("[{node_name}] start to listen strategy command");
                    self.listen_command().await;
                }

                KlineNodeAction::CancelAsyncTask => {
                    tracing::info!("[{node_name}] cancel node task");
                    self.with_ctx_read(|ctx| {
                        ctx.request_cancel();
                    })
                    .await;
                }
                KlineNodeAction::LogError(error) => {
                    tracing::error!("[{node_name}] node failed: {:?}", error);
                }
                _ => {}
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}
