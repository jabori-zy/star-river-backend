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
use types::{error::engine_error::strategy_error::NodeConfigNullSnafu, position::virtual_position::VirtualPosition};
use types::strategy::StrategyConfig;
use event_center::EventPublisher;
use tokio::sync::{Mutex, Notify};
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use petgraph::Graph;
use std::collections::HashMap;
use serde_json::Value;
use tokio_util::sync::CancellationToken;
use backtest_strategy_function::BacktestStrategyFunction;
use crate::strategy_engine::{node::BacktestNodeTrait, strategy::backtest_strategy::backtest_strategy_state_machine::*};
use types::cache::Key;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use tokio::sync::{mpsc, broadcast};
use types::strategy::node_command::NodeCommand;
use virtual_trading::VirtualTradingSystem;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use super::StrategyCommandPublisher;
use types::virtual_trading_system::event::VirtualTradingSystemEvent;
use types::order::virtual_order::VirtualOrder;
use types::strategy_stats::event::StrategyStatsEvent;
use types::strategy_stats::StatsSnapshot;
use strategy_stats::backtest_strategy_stats::BacktestStrategyStats;
use types::transaction::virtual_transaction::VirtualTransaction;
use types::error::engine_error::strategy_engine_error::strategy_error::backtest_strategy_error::BacktestStrategyError;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_log_message::StrategyStateLogMsg;
use types::strategy::node_event::LogLevel;
use event_center::strategy_event::backtest_strategy_event::{BacktestStrategyEvent, StrategyStartLogEvent};
use utils::get_utc8_timestamp_millis;



#[derive(Debug, Clone)]
pub struct BacktestStrategy {
    pub context: Arc<RwLock<BacktestStrategyContext>>,
}


impl BacktestStrategy {
    pub async fn new(
        strategy_config: StrategyConfig,
        event_publisher: EventPublisher, 
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        response_event_receiver: EventReceiver,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>
    ) -> Self {
        
        let context = BacktestStrategyContext::new(
            strategy_config,
            event_publisher,
            response_event_receiver,
            database,
            heartbeat,
            command_publisher,
            command_receiver,
        );
        Self { context: Arc::new(RwLock::new(context)) }
    }

    pub async fn add_node(
        &mut self, 
        market_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
    ) -> Result<(), BacktestStrategyError> {
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

        
        // tracing::debug!("添加节点: {:?}", node_config);

        let context = self.get_context();
        for node_config in node_config_list {
            BacktestStrategyFunction::add_node(
                context.clone(),
                node_config,
                market_event_receiver.resubscribe(),
                response_event_receiver.resubscribe(),
                node_command_tx.clone(),
                strategy_inner_event_rx.resubscribe(),
                ).await.unwrap();
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
            let transition_result = state_manager.transition(event).unwrap();
            (transition_result, state_manager)
        };

        tracing::info!("需要执行的动作: {:?}", transition_result.actions);
        for action in transition_result.actions {
            // action execute result flag
            match action {
                BacktestStrategyStateAction::InitInitialPlaySpeed => {
                    tracing::info!("{}: 初始化初始播放速度", strategy_name);
                    let context_guard = self.context.read().await;
                    let start_node_config = context_guard.get_start_node_config().await;
                    if let Ok(start_node_config) = start_node_config {
                        let mut initial_play_speed_guard = context_guard.initial_play_speed.write().await;
                        *initial_play_speed_guard = start_node_config.play_speed as u32;
                        tracing::info!("{}: 初始化初始播放速度成功。播放速度: {:?}", strategy_name, *initial_play_speed_guard);
                    } else {
                        tracing::error!("{}: 获取start节点配置失败", strategy_name);
                    }
                    
                }
                BacktestStrategyStateAction::InitSignalCount => {
                    tracing::info!("{}: 初始化信号计数", strategy_name);
                    let mut context_guard = self.context.write().await;
                    let signal_count = context_guard.get_signal_count().await;
                    if let Ok(signal_count) = signal_count {
                        let mut signal_count_guard = context_guard.total_signal_count.write().await;
                        *signal_count_guard = signal_count;
                        tracing::info!("{}: 初始化信号计数成功", strategy_name);
                    } else {
                        tracing::error!("{}: 获取信号计数失败", strategy_name);
                    }
                }
                BacktestStrategyStateAction::InitCacheLength => {
                    tracing::info!("{}: 初始化缓存长度", strategy_name);
                    let mut context_guard = self.context.write().await;
                    let cache_lengths = context_guard.get_cache_length().await;
                    if let Ok(cache_lengths) = cache_lengths {
                        context_guard.cache_lengths = cache_lengths;
                    } else {
                        tracing::error!("{}: 获取缓存长度失败", strategy_name);
                    }
                }
                BacktestStrategyStateAction::InitVirtualTradingSystem => {
                    tracing::info!("{}: 初始化虚拟交易系统", strategy_name);
                    let context_guard = self.context.read().await;
                    let virtual_trading_system = context_guard.virtual_trading_system.clone();
                    drop(context_guard); // 释放锁
                    if let Err(e) = VirtualTradingSystem::listen_play_index(virtual_trading_system).await {
                        tracing::error!("{}: 初始化虚拟交易系统失败: {}", strategy_name, e);
                    } else {
                        tracing::info!("{}: 初始化虚拟交易系统成功", strategy_name);
                    }
                }
                BacktestStrategyStateAction::InitStrategyStats => {
                    tracing::info!("{}: 初始化策略统计", strategy_name);
                    let context_guard = self.context.read().await;
                    let strategy_stats = context_guard.strategy_stats.clone();
                    drop(context_guard); // 释放锁
                    
                    if let Err(e) = BacktestStrategyStats::handle_virtual_trading_system_events(strategy_stats).await {
                        tracing::error!("{}: 初始化策略统计失败: {}", strategy_name, e);
                    } else {
                        tracing::info!("{}: 初始化策略统计成功", strategy_name);
                    }

                    // 监听播放索引
                    // BacktestStrategyStats::listen_play_index(strategy_stats2).await;
                    
                }

                BacktestStrategyStateAction::InitNode => {
                    let strategy_id = self.get_strategy_id().await;
                    tracing::info!(strategy_id = %strategy_id, strategy_name = %strategy_name, "start init node");
                    
                    // business logic is in context, here only to get the lock
                    if let Err(e) = BacktestStrategyContext::init_node(self.context.clone()).await {
                        tracing::error!("{}", e);
                        return Err(e);
                    }
                    
                    tracing::info!("{}: 所有节点已成功初始化", strategy_name);
                }

                BacktestStrategyStateAction::StopNode => {
                    tracing::info!("++++++++++++++++++++++++++++++++++++++");
                    tracing::info!("{}: 开始停止节点", strategy_name);
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
                        tracing::info!("{}: 所有节点已成功停止", strategy_name);
                    } else {
                        tracing::error!("{}: 部分节点停止失败，策略无法正常运行", strategy_name);
                    }
                }
                
                BacktestStrategyStateAction::LogTransition => {
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", strategy_name, self.get_state_machine().await.current_state(), transition_result.new_state);
                }

                BacktestStrategyStateAction::ListenAndHandleNodeEvent => {
                    tracing::info!("{}: 监听节点消息", strategy_name);
                    BacktestStrategyFunction::listen_node_events(self.get_context()).await;
                }
                BacktestStrategyStateAction::ListenAndHandleNodeCommand => {
                    tracing::info!("{}: 监听命令", strategy_name);
                    BacktestStrategyFunction::listen_node_command(self.get_context()).await;
                }
                BacktestStrategyStateAction::ListenAndHandleStrategyStatsEvent => {
                    tracing::info!("{}: 监听策略统计事件", strategy_name);
                    BacktestStrategyFunction::listen_strategy_stats_event(self.get_context()).await;
                }
                BacktestStrategyStateAction::LogError(error) => {
                    tracing::error!("{}: {}", strategy_name, error);
                }
                BacktestStrategyStateAction::LogStrategyState => {
                    tracing::info!("{}: 记录策略状态", strategy_name);

                    let log_message = StrategyStateLogMsg::new(strategy_id, strategy_name.clone(), self.get_state_machine().await.current_state().to_string());
                    let log_event = StrategyStartLogEvent {
                        strategy_id: strategy_id,
                        strategy_name: strategy_name.clone(),
                        strategy_state: Some(self.get_state_machine().await.current_state().to_string()),
                        strategy_state_action: Some(BacktestStrategyStateAction::LogStrategyState.to_string()),
                        log_level: LogLevel::Info,
                        error_code: None,
                        message: log_message.to_string(),
                        detail: None,
                        duration: None,
                        timestamp: get_utc8_timestamp_millis(),
                    };
                    let backtest_strategy_event = BacktestStrategyEvent::StrategyStartLog(log_event.clone());
                    let _ = self.get_context().read().await.get_event_publisher().publish(backtest_strategy_event.into()).await;
                }
            };

            {
                let mut context_guard = self.context.write().await;
                context_guard.set_state_machine(state_machine.clone());
            }
            

            
        }
        Ok(())
        

    }

    
    pub async fn init_strategy(&mut self) -> Result<(), BacktestStrategyError> {
        tracing::info!("{}: 开始初始化策略", self.get_strategy_name().await);

        // created => initializing
        self.update_strategy_state(BacktestStrategyStateTransitionEvent::Initialize).await?;

        // 
        // initializing => ready
        tracing::info!("{}: 初始化完成", self.get_strategy_name().await);
        self.update_strategy_state(BacktestStrategyStateTransitionEvent::InitializeComplete).await?;

        Ok(())
    }

    pub async fn stop_strategy(&mut self) -> Result<(), String> {
        // 获取当前状态
        // 如果策略当前状态为 Stopped，则不进行操作
        let current_state = self.get_state_machine().await.current_state();
        if current_state == BacktestStrategyRunState::Stopping {
            tracing::info!("策略{}已停止", self.get_strategy_name().await);
            return Ok(());
        }
        tracing::info!("等待所有节点停止...");
        self.update_strategy_state(BacktestStrategyStateTransitionEvent::Stop).await.unwrap();

        // 发送完信号后，循环遍历所有的节点，获取节点的状态，如果所有的节点状态都为stopped，则更新策略状态为Stopped
        let all_stopped = {
            let context_guard = self.context.read().await;
            context_guard.wait_for_all_nodes_stopped(10).await.unwrap()
        };
        if all_stopped {
            self.update_strategy_state(BacktestStrategyStateTransitionEvent::StopComplete).await.unwrap();
            Ok(())
        } else {
            Err("等待节点停止超时".to_string())
        }
    }

    pub async fn play(&mut self) -> Result<(), String> {
        tracing::info!("{}: 开始播放k线", self.get_strategy_name().await);
        let context_guard = self.context.read().await;
        context_guard.play().await;
        Ok(())
    }

    pub async fn pause(&mut self) -> Result<(), String> {
        let mut context_guard = self.context.write().await;
        context_guard.pause().await;
        Ok(())
    }

    pub async fn reset(&mut self) -> Result<(), String> {
        tracing::info!("{}: 重置播放", self.get_strategy_name().await);
        let mut context_guard = self.context.write().await;
        context_guard.reset().await;
        // 重置虚拟交易系统
        context_guard.virtual_trading_system_reset().await;
        // 重置策略统计
        context_guard.strategy_stats_reset().await;
        context_guard.send_reset_node_event().await;
        
        Ok(())
    }

    pub async fn play_one_kline(&mut self) -> Result<i32, String> {
        
        let context_guard = self.context.read().await;
        let play_index = context_guard.play_one_kline().await;
        if let Ok(play_index) = play_index {
            Ok(play_index)
        } else {
            Err("播放单根k线失败".to_string())
        }
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