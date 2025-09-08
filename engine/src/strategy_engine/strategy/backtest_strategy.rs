pub mod backtest_strategy_context;
pub mod backtest_strategy_state_machine;
pub mod backtest_strategy_function;
pub mod backtest_strategy_control;
pub mod backtest_strategy_log_message;


use std::sync::Arc;
use tokio::sync::RwLock;
use backtest_strategy_context::BacktestStrategyContext;
use backtest_strategy_state_machine::{BacktestStrategyStateAction, BacktestStrategyStateMachine};
use types::error::engine_error::strategy_error::EdgeConfigNullSnafu;
use types::error::error_trait::{StarRiverErrorTrait, Language};
use types::{error::engine_error::strategy_error::NodeConfigNullSnafu, position::virtual_position::VirtualPosition};
use types::strategy::StrategyConfig;
use tokio::sync::Mutex;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use backtest_strategy_function::BacktestStrategyFunction;
use crate::strategy_engine::{node::BacktestNodeTrait, strategy::backtest_strategy::backtest_strategy_state_machine::*};
use tokio::sync::{mpsc, broadcast};
use types::strategy::node_command::NodeCommand;
use virtual_trading::VirtualTradingSystem;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use types::order::virtual_order::VirtualOrder;
use types::strategy_stats::StatsSnapshot;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use types::transaction::virtual_transaction::VirtualTransaction;
use types::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::BacktestStrategyError;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_log_message::StrategyStateLogMsg;
use types::strategy::node_event::LogLevel;
use event_center::strategy_event::backtest_strategy_event::{BacktestStrategyEvent, StrategyStateLogEvent};
use utils::get_utc8_timestamp_millis;
use snafu::IntoError;
use types::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::*;
use event_center::EventCenterSingleton;




#[derive(Debug, Clone)]
pub struct BacktestStrategy {
    pub context: Arc<RwLock<BacktestStrategyContext>>,
}


impl BacktestStrategy {
    pub async fn new(
        strategy_config: StrategyConfig,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>
    ) -> Self {
        
        let context = BacktestStrategyContext::new(
            strategy_config,
            database,
            heartbeat,
        );
        Self { context: Arc::new(RwLock::new(context)) }
    }


    pub async fn add_node(&mut self) -> Result<(), BacktestStrategyError> {
        let (node_command_tx, node_command_rx) = mpsc::channel::<NodeCommand>(100);
        let (strategy_inner_event_tx, strategy_inner_event_rx) = broadcast::channel::<StrategyInnerEvent>(100);


        // setting strategy context properties
        {
            let mut context_guard = self.context.write().await;
            context_guard.set_node_command_receiver(node_command_rx);
            context_guard.set_strategy_inner_event_publisher(strategy_inner_event_tx);

        }// context lock end

        // get strategy config
        let node_config_list = {
            let context_guard = self.context.read().await;
            let node_config_list = context_guard.strategy_config.nodes
            .as_ref()
            .and_then(|node| node.as_array())
            .ok_or_else(|| NodeConfigNullSnafu {
                strategy_id: context_guard.strategy_id,
                strategy_name: context_guard.strategy_name.clone(),
            }.build())?;
            node_config_list.clone()
        };


        let context = self.get_context();
        for node_config in node_config_list {
            let result = BacktestStrategyFunction::add_node(
                context.clone(),
                node_config,
                node_command_tx.clone(),
                strategy_inner_event_rx.resubscribe(),
                ).await;
            if let Err(e) = result {
                let error = NodeCheckSnafu {}.into_error(e);
                return Err(error);
            }
            }

        Ok(())
    }

    pub async fn add_edge(&mut self) -> Result<(), BacktestStrategyError> {
        let context = self.get_context();
        
        let edge_config_list = {
            let context_guard = context.read().await;
            context_guard.strategy_config.edges
            .as_ref()
            .and_then(|edge| edge.as_array())
            .ok_or_else(|| EdgeConfigNullSnafu {
                strategy_id: context_guard.strategy_id,
                strategy_name: context_guard.strategy_name.clone(),
            }.build())?.clone()
        };

        for edge_config in edge_config_list {
            BacktestStrategyFunction::add_edge(
                context.clone(),
                edge_config,
            ).await.unwrap();
        }
        Ok(())
    }

    pub async fn set_leaf_nodes(&mut self) -> Result<(), BacktestStrategyError> {
        let context = self.get_context();
        BacktestStrategyFunction::set_leaf_nodes(context).await;
        Ok(())
    }

    pub async fn set_strategy_output_handles(&mut self) -> Result<(), BacktestStrategyError> {
        let context = self.get_context();
        BacktestStrategyFunction::add_strategy_output_handle(context).await;
        Ok(())
    }

    
}


impl BacktestStrategy {

    pub fn get_context(&self) -> Arc<RwLock<BacktestStrategyContext>> {
        self.context.clone()
    }

    pub async fn get_strategy_id(&self) -> i32 {
        self.context.read().await.strategy_id
    }

    pub async fn get_strategy_name(&self) -> String {
        self.context.read().await.strategy_name.clone()
    }

    pub async fn get_state_machine(&self) -> BacktestStrategyStateMachine {
        self.context.read().await.state_machine.clone()
    }

    pub async fn update_strategy_state(&mut self, event: BacktestStrategyStateTransitionEvent) -> Result<(), BacktestStrategyError> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;

        let (transition_result, state_machine) = {
            let mut state_manager = self.context.read().await.state_machine.clone();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        for action in transition_result.get_actions() {
            // action execute result flag
            match action {
                BacktestStrategyStateAction::InitInitialPlaySpeed => {
                    tracing::info!("[{}({})] init initial play speed", strategy_name, strategy_id);
                    let context_guard = self.context.read().await;
                    let start_node_config = context_guard.get_start_node_config().await;
                    if let Ok(start_node_config) = start_node_config {
                        let mut initial_play_speed_guard = context_guard.initial_play_speed.write().await;
                        *initial_play_speed_guard = start_node_config.play_speed as u32;
                        tracing::info!("[{}({})] init initial play speed success. initial play speed: {:?}", strategy_name, strategy_id, *initial_play_speed_guard);
                    } else {
                        tracing::error!("[{}({})] get start node config failed", strategy_name, strategy_id);
                    }
                    
                }
                BacktestStrategyStateAction::InitSignalCount => {
                    tracing::info!("[{}({})] init signal count", strategy_name, strategy_id);
                    let mut context_guard = self.context.write().await;
                    let signal_count = context_guard.get_signal_count().await;
                    if let Ok(signal_count) = signal_count {
                        let mut signal_count_guard = context_guard.total_signal_count.write().await;
                        *signal_count_guard = signal_count;
                        tracing::info!("[{}({})] init signal count success", strategy_name, strategy_id);
                    } else {
                        tracing::error!("[{}({})] get signal count failed", strategy_name, strategy_id);
                    }
                }
                BacktestStrategyStateAction::InitCacheLength => {
                    tracing::info!("[{}({})] init cache length", strategy_name, strategy_id);
                    let mut context_guard = self.context.write().await;
                    let cache_lengths = context_guard.get_cache_length().await;
                    if let Ok(cache_lengths) = cache_lengths {
                        context_guard.cache_lengths = cache_lengths;
                    } else {
                        tracing::error!("[{}({})] get cache length failed", strategy_name, strategy_id);
                    }
                }
                BacktestStrategyStateAction::InitVirtualTradingSystem => {
                    tracing::info!("[{}({})] init virtual trading system", strategy_name, strategy_id);
                    let context_guard = self.context.read().await;
                    let virtual_trading_system = context_guard.virtual_trading_system.clone();
                    drop(context_guard); // 释放锁
                    if let Err(e) = VirtualTradingSystem::listen_play_index(virtual_trading_system).await {
                        tracing::error!("[{}({})] init virtual trading system failed: {}", strategy_name, strategy_id, e);
                    } else {
                        tracing::info!("[{}({})] init virtual trading system success", strategy_name, strategy_id);
                    }
                }
                BacktestStrategyStateAction::InitStrategyStats => {
                    tracing::info!("[{}({})] init strategy stats", strategy_name, strategy_id);
                    let context_guard = self.context.read().await;
                    let strategy_stats = context_guard.strategy_stats.clone();
                    drop(context_guard); // 释放锁
                    
                    if let Err(e) = BacktestStrategyStats::handle_virtual_trading_system_events(strategy_stats).await {
                        tracing::error!("[{}({})] init strategy stats failed: {}", strategy_name, strategy_id, e);
                    } else {
                        tracing::info!("[{}({})] init strategy stats success", strategy_name, strategy_id);
                    }
                    
                    
                }

                BacktestStrategyStateAction::CheckNode => {
                    let (strategy_id, strategy_name, current_state) = {
                        let context_guard = self.context.read().await;
                        let strategy_id = context_guard.strategy_id;
                        let strategy_name = context_guard.strategy_name.clone();
                        let current_state = self.get_state_machine().await.current_state();
                        (strategy_id, strategy_name, current_state)
                    };
                    
                    let add_node_result = self.add_node().await;

                    // let log_message = StrategyStartLogMsg::new(strategy_id, strategy_name.clone(), current_state.to_string(), BacktestStrategyStateAction::CheckNode.to_string(), e.to_string());
                    if let Err(e) = add_node_result {
                        let error_message = e.get_error_message(Language::Chinese);
                        let log_event = StrategyStateLogEvent {
                            strategy_id,
                            strategy_name,
                            strategy_state: Some(current_state.to_string()),
                            strategy_state_action: Some(BacktestStrategyStateAction::CheckNode.to_string()),
                            error_code: Some(e.error_code()),
                            error_code_chain: Some(e.error_code_chain()),
                            message: error_message,
                            timestamp: get_utc8_timestamp_millis(),
                            log_level: LogLevel::Error,
                        };
                        let backtest_strategy_event = BacktestStrategyEvent::StrategyStateLog(log_event.clone());
                        tracing::debug!("publish strategy start log event: {:?}", backtest_strategy_event);
                        let _ = EventCenterSingleton::publish(backtest_strategy_event.into()).await;
                        return Err(e);
                    }
                    self.add_edge().await?;
                    self.set_leaf_nodes().await?;
                    self.set_strategy_output_handles().await?;

                }

                BacktestStrategyStateAction::InitNode => {
                    let strategy_id = self.get_strategy_id().await;
                    tracing::info!("[{}({})] start init node", strategy_name, strategy_id);
                    
                    // business logic is in context, here only to get the lock
                    if let Err(e) = BacktestStrategyContext::init_node(self.context.clone()).await {
                        tracing::error!("{}", e);
                        return Err(e);
                    }
                    
                    tracing::info!("[{}] all nodes initialized.", strategy_name);
                }

                BacktestStrategyStateAction::StopNode => {
                    tracing::info!("[{}] start stop node", strategy_name);
                    let nodes = {
                        let context_guard = self.context.read().await;
                        context_guard.topological_sort()
                    };
                    
                    let mut all_nodes_stopped = true;

                    for node in nodes {
                        // let mut node = node.clone();
                        let context_guard = self.context.read().await;
                        
                        if let Err(e) = context_guard.stop_node(node).await {
                            tracing::error!("{}", e);
                            all_nodes_stopped = false;
                            break;
                        }
                    }

                    if all_nodes_stopped {
                        tracing::info!("[{}] all nodes stopped", strategy_name);
                    } else {
                        tracing::error!("[{}] some nodes stop failed, strategy cannot run normally", strategy_name);
                    }
                }
                
                BacktestStrategyStateAction::LogTransition => {
                    
                    tracing::debug!("[{}] state transition: {:?} -> {:?}", strategy_name, self.get_state_machine().await.current_state(), transition_result.get_new_state());
                }

                BacktestStrategyStateAction::ListenAndHandleNodeEvent => {
                    tracing::info!("[{}] listen node events", strategy_name);
                    BacktestStrategyFunction::listen_node_events(self.get_context()).await;
                }
                BacktestStrategyStateAction::ListenAndHandleNodeCommand => {
                    tracing::info!("[{}] listen node command", strategy_name);
                    BacktestStrategyFunction::listen_node_command(self.get_context()).await;
                }
                BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent => {
                    tracing::info!("[{}] listen strategy stats event", strategy_name);
                    BacktestStrategyFunction::listen_strategy_stats_event(self.get_context()).await;
                }
                BacktestStrategyStateAction::LogError(error) => {
                    tracing::error!("[{}] {}", strategy_name, error);
                }
                BacktestStrategyStateAction::LogStrategyState => {
                    let current_state = self.get_state_machine().await.current_state();

                    let log_message = StrategyStateLogMsg::new(strategy_id, strategy_name.clone(), current_state.to_string());
                    let log_event = StrategyStateLogEvent {
                        strategy_id,
                        strategy_name: strategy_name.clone(),
                        strategy_state: Some(current_state.to_string()),
                        strategy_state_action: Some(BacktestStrategyStateAction::LogStrategyState.to_string()),
                        log_level: LogLevel::Info,
                        error_code: None,
                        error_code_chain: None,
                        message: log_message.to_string(),
                        timestamp: get_utc8_timestamp_millis(),
                    };
                    let backtest_strategy_event = BacktestStrategyEvent::StrategyStateLog(log_event.clone());
                    // let _ = self.get_context().read().await.get_event_publisher().publish(backtest_strategy_event.into()).await;
                    let _ = EventCenterSingleton::publish(backtest_strategy_event.into()).await;

                }
            };

            {
                let mut context_guard = self.context.write().await;
                context_guard.set_state_machine(state_machine.clone());
            }
            

            
        }
        Ok(())
        

    }

    pub async fn check_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;
        
        tracing::info!("[{}({})] starting check strategy", strategy_name, strategy_id);
        self.context.write().await.update_strategy_status(BacktestStrategyRunState::Checking.to_string().to_lowercase()).await?;
        
        let update_result = self.update_strategy_state(BacktestStrategyStateTransitionEvent::Check).await;
        if let Err(e) = update_result {
            self.context.write().await.update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase()).await?;
            return Err(e);
        }

        tracing::info!("[{}({})] check finished.", strategy_name, strategy_id);
        self.context.write().await.update_strategy_status(BacktestStrategyRunState::CheckPassed.to_string().to_lowercase()).await?;
        let update_result = self.update_strategy_state(BacktestStrategyStateTransitionEvent::CheckComplete).await;
        if let Err(e) = update_result {
            self.context.write().await.update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase()).await?;
            return Err(e);
        }
        Ok(())

    }

    
    pub async fn init_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;
        tracing::info!("[{}({})] starting init strategy", strategy_name, strategy_id);

        // created => initializing
        self.context.write().await.update_strategy_status(BacktestStrategyRunState::Initializing.to_string().to_lowercase()).await?;
        let update_result = self.update_strategy_state(BacktestStrategyStateTransitionEvent::Initialize).await;
        if let Err(e) = update_result {
            self.context.write().await.update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase()).await?;
            return Err(e);
        }

        // 
        // initializing => ready
        tracing::info!("[{}({})] init finished.", strategy_name, strategy_id);
        self.context.write().await.update_strategy_status(BacktestStrategyRunState::Ready.to_string().to_lowercase()).await?;
        let update_result = self.update_strategy_state(BacktestStrategyStateTransitionEvent::InitializeComplete).await;
        if let Err(e) = update_result {
            self.context.write().await.update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase()).await?;
            return Err(e);
        }

        Ok(())
    }

    pub async fn stop_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        let current_state = self.get_state_machine().await.current_state();
        if current_state == BacktestStrategyRunState::Stopping {
            tracing::info!("[{}({})] stopped.", self.get_strategy_name().await, self.get_strategy_id().await);
            return Ok(());
        }
        tracing::info!("waiting for all nodes to stop...");
        self.context.write().await.update_strategy_status(BacktestStrategyRunState::Stopping.to_string().to_lowercase()).await?;
        let update_result = self.update_strategy_state(BacktestStrategyStateTransitionEvent::Stop).await;
        if let Err(e) = update_result {
            self.context.write().await.update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase()).await?;
            return Err(e);
        }

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = {
            let context_guard = self.context.read().await;
            context_guard.wait_for_all_nodes_stopped(10).await.unwrap()
        };
        if all_stopped {
            self.context.write().await.update_strategy_status(BacktestStrategyRunState::Stopped.to_string().to_lowercase()).await?;
            let update_result = self.update_strategy_state(BacktestStrategyStateTransitionEvent::StopComplete).await;
            if let Err(e) = update_result {
                self.context.write().await.update_strategy_status(BacktestStrategyRunState::Failed.to_string().to_lowercase()).await?;
                return Err(e);
            }
            Ok(())
        } else {
            Err(WaitAllNodesStoppedTimeoutSnafu {}.build())
        }
    }

    pub async fn play(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;
        tracing::info!("[{}({})] start play kline", strategy_name, strategy_id);
        let mut context_guard = self.context.write().await;
        context_guard.play().await?;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), BacktestStrategyError> {
        let mut context_guard = self.context.write().await;
        context_guard.pause().await?;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), BacktestStrategyError> {
        let strategy_name = self.get_strategy_name().await;
        let strategy_id = self.get_strategy_id().await;
        tracing::info!("[{}({})] reset play", strategy_name, strategy_id);
        let mut context_guard = self.context.write().await;
        context_guard.reset().await?;
        // 重置虚拟交易系统
        context_guard.virtual_trading_system_reset().await;
        // 重置策略统计
        context_guard.strategy_stats_reset().await;
        context_guard.send_reset_node_event().await;
        
        Ok(())
    }

    pub async fn play_one_kline(&mut self) -> Result<i32, BacktestStrategyError> {
        
        let context_guard = self.context.read().await;
        let play_index = context_guard.play_one_kline().await?;
        Ok(play_index)
    }

    pub async fn get_play_index(&self) -> i32 {
        let context_guard = self.context.read().await;
        context_guard.get_play_index().await
    }

    pub async fn get_virtual_orders(&self) -> Vec<VirtualOrder> {
        let context_guard = self.context.read().await;
        context_guard.get_virtual_orders().await
    }

    pub async fn get_current_positions(&self) -> Vec<VirtualPosition> {
        let context_guard = self.context.read().await;
        context_guard.get_current_positions().await
    }

    pub async fn get_history_positions(&self) -> Vec<VirtualPosition> {
        let context_guard = self.context.read().await;
        context_guard.get_history_positions().await
    }

    pub async fn get_transactions(&self) -> Vec<VirtualTransaction> {
        let context_guard = self.context.read().await;
        context_guard.get_transactions().await
    }

    pub async fn get_stats_history(&self, play_index: i32) -> Vec<StatsSnapshot> {
        let context_guard = self.context.read().await;
        context_guard.get_stats_history(play_index).await
    }
    
}