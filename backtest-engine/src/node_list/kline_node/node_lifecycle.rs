use async_trait::async_trait;
use crate::node::node_trait::{NodeContextAccessor, NodeLifecycle, NodeEventListener};
use crate::node::node_context_trait::{NodeStateMachineTrait, NodeIdentity, NodeHandleTrait, NodeControl};
use super::{KlineNode, KlineNodeContext};
use crate::node_list::kline_node::state_machine::KlineNodeAction;
use crate::error::node_error::BacktestNodeError;
use crate::node::node_state_machine::NodeStateTransTrigger;
use snafu::{Report, IntoError};
use crate::node::node_message::{common_log_message::*, kline_node_log_message::*};
use crate::node::node_utils::NodeUtils;
use crate::error::node_error::kline_node_error::RegisterExchangeFailedSnafu;
use event_center::communication::Response;
use event_center::event::strategy_event::NodeStateLogEvent;
use crate::node::node_state_machine::NodeRunState;

#[async_trait]
impl NodeLifecycle<KlineNodeContext, KlineNodeAction> for KlineNode {
    async fn init(&self) -> Result<(), BacktestNodeError> {
        let node_name = self.with_ctx_read(|ctx| {
            ctx.node_name().to_string()
        }).await;
        tracing::info!("================={}====================", node_name);
        tracing::info!("[{node_name}] start init");
        // 开始初始化 created -> Initialize
        if let Err(error) = self.update_node_state(NodeStateTransTrigger::StartInit).await {
            let report = Report::from_error(&error);
            tracing::error!("report: {}", report.to_string());
            return Err(error);
        }

        let current_state = self.with_ctx_read_async(|ctx| {
            Box::pin(async move {
                ctx.run_state().await.clone()
            })
        }).await;
        
        tracing::info!("[{node_name}] init complete: {:?}", current_state);
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(NodeStateTransTrigger::FinishInit).await?;
        Ok(())
    }

    async fn stop(&self) -> Result<(), BacktestNodeError> {
        let node_name = self.with_ctx_read(|ctx| {
            ctx.node_name().to_string()
        }).await;
        tracing::info!("=================stop node [{node_name}]====================");
        tracing::info!("[{node_name}] start to stop");
        self.update_node_state(NodeStateTransTrigger::StartStop).await?;
        // 切换为stopped状态
        self.update_node_state(NodeStateTransTrigger::FinishStop).await?;
        Ok(())
    }

    async fn update_node_state(&self, event: NodeStateTransTrigger) -> Result<(), BacktestNodeError> {
        let (node_name, node_id, strategy_id, strategy_output_handle, mut state_machine) = self
            .with_ctx_read(|ctx| {
                let node_name = ctx.node_name().to_string();
                let node_id = ctx.node_id().clone();
                let strategy_id = ctx.strategy_id().clone();
                let strategy_output_handle = ctx.strategy_output_handle().clone();
                let state_machine = ctx.state_machine().clone();
                (node_name, node_id, strategy_id, strategy_output_handle, state_machine)
        }).await;

        let transition_result = {
            let mut state_machine = state_machine.write().await;
            state_machine.transition(event)?
        };

        // 执行转换后需要执行的动作
        for action in transition_result.actions() {
            // 克隆actions避免移动问题
            let current_state = {
                let state_machine = state_machine.read().await;
                state_machine.current_state().clone()
            };

            match action {
                KlineNodeAction::LogTransition => {
                    tracing::debug!(
                        "[{node_name}] state transition: {:?} -> {:?}",
                        current_state,
                        transition_result.new_state()
                    );
                }
                KlineNodeAction::LogNodeState => {
                    let log_message = NodeStateLogMsg::new(node_name.clone(), current_state.to_string());
                    NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeAction::LogNodeState.to_string(), &strategy_output_handle).await;
                }
                KlineNodeAction::ListenAndHandleExternalEvents => {
                    tracing::info!("[{node_name}] starting to listen external events");
                    let log_message = ListenExternalEventsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeAction::ListenAndHandleExternalEvents.to_string(), &strategy_output_handle).await;
                    
                    self.listen_external_event().await;
                }
                KlineNodeAction::ListenAndHandleNodeEvents => {
                    tracing::info!("[{node_name}] starting to listen node events");
                    let log_message = ListenNodeEventsMsg::new(node_name.clone());
                    NodeUtils::send_success_status_event(strategy_id, node_id.clone(), node_name.clone(), log_message.to_string(), current_state.to_string(), KlineNodeAction::ListenAndHandleNodeEvents.to_string(), &strategy_output_handle).await;
                    
                    self.listen_source_node_events().await;
                }
                KlineNodeAction::GetMinIntervalSymbols => {
                    tracing::info!("[{node_name}] start to get min interval symbols");
                    let result = self
                        .with_ctx_write_async(|ctx| {
                            Box::pin(async move {
                                ctx
                                .get_min_interval_symbols()
                                .await
                                .map(|min_interval_symbols| {
                                        ctx.set_min_interval_symbols(min_interval_symbols);
                                    })
                            })
                        })
                        .await;

                    match result {
                        Ok(()) => {
                            let log_message = GetMinIntervalSymbolsSuccessMsg::new(node_name.clone());
                            NodeUtils::send_success_status_event(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                log_message.to_string(),
                                current_state.to_string(),
                                KlineNodeAction::GetMinIntervalSymbols.to_string(),
                                &strategy_output_handle,
                            )
                            .await;
                        }
                        Err(err) => {
                            tracing::warn!(
                                "[{node_name}] failed to get min interval symbols from strategy: {err}"
                            );
                        }
                    }
                }
                KlineNodeAction::RegisterExchange => {
                    tracing::info!("[{node_name}] start to register exchange");

                    let (exchange, response) = self.with_ctx_write_async(|ctx| {
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
                    }).await;

                    
                    // 发送开始注册日志
                    let log_message = StartRegisterExchangeMsg::new(node_name.clone(), exchange.clone());
                    NodeUtils::send_success_status_event(
                        strategy_id,
                        node_id.clone(),
                        node_name.clone(),
                        log_message.to_string(),
                        current_state.to_string(),
                        KlineNodeAction::RegisterExchange.to_string(),
                        &strategy_output_handle
                    ).await;

                    if response.is_success() {
                        let log_message = RegisterExchangeSuccessMsg::new(node_name.clone(), exchange);
                        NodeUtils::send_success_status_event(
                            strategy_id,
                            node_id.clone(),
                            node_name.clone(),
                            log_message.to_string(),
                            current_state.to_string(),
                            KlineNodeAction::RegisterExchange.to_string(),
                            &strategy_output_handle
                        ).await;
                    } else {
                        // 转换状态 Failed
                        let error = response.get_error();
                        let kline_error = RegisterExchangeFailedSnafu {
                            node_name: node_name.clone(),
                        }
                        .into_error(error.clone());

                        let log_event = NodeStateLogEvent::error(
                            strategy_id.clone(),
                            node_id.clone(),
                            node_name.clone(),
                            NodeRunState::Failed.to_string(),
                            KlineNodeAction::RegisterExchange.to_string(),
                            &kline_error,
                        );
                        let _ = strategy_output_handle.send(log_event.into());
                        return Err(kline_error.into());
                    }
                }
                KlineNodeAction::LoadHistoryFromExchange => {
                    tracing::info!("[{node_name}] starting to load kline data from exchange");
                    let load_result = self
                        .with_ctx_write_async(|ctx| {
                            Box::pin(async move { ctx.load_kline_history_from_exchange().await })
                        })
                        .await;

                    match load_result {
                        Ok(()) => {
                            tracing::info!("[{node_name}] load kline history from exchange success");
                            let log_message = LoadKlineDataSuccessMsg::new(node_name.clone());
                            NodeUtils::send_success_status_event(
                                strategy_id,
                                node_id.clone(),
                                node_name.clone(),
                                log_message.to_string(),
                                current_state.to_string(),
                                KlineNodeAction::LoadHistoryFromExchange.to_string(),
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
                                KlineNodeAction::LoadHistoryFromExchange.to_string(),
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
                    self.listen_node_command().await;
                }

                KlineNodeAction::CancelAsyncTask => {
                    tracing::info!("[{node_name}] cancel node task");
                    self.with_ctx_read(|ctx| {
                        ctx.request_cancel();
                    }).await;
                }
                KlineNodeAction::LogError(error) => {
                    tracing::error!("[{node_name}] node failed: {:?}", error);
                }
                _ => {}
            }
            
            // // 动作执行完毕后更新节点最新的状态
            // {
            //     self.context.write().await.set_state_machine(state_machine.clone_box());
            // }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        Ok(())
    }
}