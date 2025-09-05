
mod futures_order_node_state_machine;
mod futures_order_node_context;
pub mod futures_order_node_types;


use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;
use super::futures_order_node::futures_order_node_state_machine::{OrderNodeStateMachine, OrderNodeStateAction};
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use std::time::Duration;
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType};
use super::futures_order_node::futures_order_node_context::FuturesOrderNodeContext;
use futures_order_node_types::*;
use sea_orm::DatabaseConnection;
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use event_center::command::backtest_strategy_command::StrategyCommandReceiver;
use types::strategy::node_command::NodeCommandSender;
use crate::strategy_engine::node::node_state_machine::*;
use virtual_trading::VirtualTradingSystem;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use types::strategy::node_event::BacktestNodeEvent;
use types::order::OrderType;
use tokio_stream::wrappers::BroadcastStream;
use futures::StreamExt;
use std::collections::HashMap;
use snafu::ResultExt;
use types::virtual_trading_system::event::VirtualTradingSystemEventReceiver;
use types::custom_type::{NodeId, NodeName, PlayIndex, StrategyId};
use types::error::engine_error::node_error::futures_order_node_error::ConfigFieldValueNullSnafu;
use types::error::engine_error::strategy_engine_error::node_error::*;
use types::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::futures_order_node_error::*;

#[derive(Debug, Clone)]
pub struct FuturesOrderNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,

}

impl FuturesOrderNode {
    pub fn new(
        node_config: serde_json::Value,
        database: DatabaseConnection,
        heartbeat: Arc<Mutex<Heartbeat>>,
        node_command_sender: NodeCommandSender,
        strategy_command_receiver: Arc<Mutex<StrategyCommandReceiver>>,
        virtual_trading_system: Arc<Mutex<VirtualTradingSystem>>,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
        virtual_trading_system_event_receiver: VirtualTradingSystemEventReceiver,
        play_index_watch_rx: tokio::sync::watch::Receiver<PlayIndex>,
    ) -> Result<Self, FuturesOrderNodeError> {
        let (strategy_id, node_id, node_name, backtest_config) = Self::check_futures_order_node_config(node_config)?;
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::OrderNode,
            Box::new(OrderNodeStateMachine::new(node_id, node_name)),
            node_command_sender,
            strategy_command_receiver,
            strategy_inner_event_receiver,
            play_index_watch_rx,
        );
        Ok(Self {
            context: Arc::new(RwLock::new(Box::new(FuturesOrderNodeContext {
                base_context,
                backtest_config,
                is_processing_order: Arc::new(RwLock::new(HashMap::new())),
                database,
                heartbeat,
                virtual_trading_system,
                virtual_trading_system_event_receiver,
                unfilled_virtual_order: Arc::new(RwLock::new(HashMap::new())),
                virtual_order_history: Arc::new(RwLock::new(HashMap::new())),
                virtual_transaction_history: Arc::new(RwLock::new(HashMap::new())),
                min_kline_interval: None,
            }))),
        })
    }

    fn check_futures_order_node_config(node_config: serde_json::Value) -> Result<(StrategyId, NodeId, NodeName, FuturesOrderNodeBacktestConfig), FuturesOrderNodeError> {
        let node_id = node_config
            .get("id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| ConfigFieldValueNullSnafu {field_name: "id".to_string()}.build())?
            .to_owned();
        let node_data = node_config
            .get("data")
            .ok_or_else(|| ConfigFieldValueNullSnafu {field_name: "data".to_string()}.build())?
            .to_owned();
        let node_name = node_data
            .get("nodeName")
            .and_then(|name| name.as_str())
            .ok_or_else(|| ConfigFieldValueNullSnafu {field_name: "nodeName".to_string()}.build())?
            .to_owned();
        let strategy_id = node_data
            .get("strategyId")
            .and_then(|id| id.as_i64())
            .ok_or_else(|| ConfigFieldValueNullSnafu {field_name: "strategyId".to_string()}.build())?
            .to_owned() as StrategyId;

        let backtest_config_json = node_data.get("backtestConfig")
            .ok_or_else(|| ConfigFieldValueNullSnafu {field_name: "backtestConfig".to_string()}.build())?
            .to_owned();

        let backtest_config = serde_json::from_value::<FuturesOrderNodeBacktestConfig>(backtest_config_json).context(ConfigDeserializationFailedSnafu {})?;
        Ok((strategy_id, node_id, node_name, backtest_config))
    }

    async fn listen_virtual_trading_system_events(&self) -> Result<(), String> {
        let (virtual_trading_system_event_receiver, cancel_token, node_id) = {
                let context = self.get_context();
                let context_guard = context.read().await;
                let futures_order_node_context = context_guard.as_any().downcast_ref::<FuturesOrderNodeContext>().unwrap();

                let receiver = futures_order_node_context.virtual_trading_system_event_receiver.resubscribe();
                let cancel_token = futures_order_node_context.get_cancel_token().clone();
                let node_id = futures_order_node_context.get_node_id().clone();
                (receiver, cancel_token, node_id)
            };
    
    
            // 创建一个流，用于接收节点传递过来的message
            let mut stream = BroadcastStream::new(virtual_trading_system_event_receiver);
            let context = self.get_context();
            // 节点接收数据
            tracing::info!(node_id = %node_id, "开始监听虚拟交易系统事件。");
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        // 如果取消信号被触发，则中止任务
                        _ = cancel_token.cancelled() => {
                            tracing::info!("{} 虚拟交易系统事件监听任务已中止", node_id);
                            break;
                        }
                        // 接收消息
                        receive_result = stream.next() => {
                            match receive_result {
                                Some(Ok(event)) => {
                                    // tracing::debug!("{} 收到消息: {:?}", node_id, message);
                                    let mut context_guard = context.write().await;
                                    let futures_order_node_context = context_guard.as_any_mut().downcast_mut::<FuturesOrderNodeContext>().unwrap();
                                    futures_order_node_context.handle_virtual_trading_system_event(event).await.unwrap();
                                }
                                Some(Err(e)) => {
                                    tracing::error!("节点{}接收消息错误: {}", node_id, e);
                                }
                                None => {
                                    tracing::warn!("节点{}所有消息流已关闭", node_id);
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        
        Ok(())
    }
    
}



#[async_trait]
impl BacktestNodeTrait for FuturesOrderNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }

    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }

    async fn set_output_handle(&mut self) {
        tracing::debug!("{}: 设置节点默认出口", self.get_node_id().await);
        let node_id = self.get_node_id().await;
        let node_name = self.get_node_name().await;
        let (tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
        let strategy_output_handle_id = format!("{}_strategy_output", node_id);
        tracing::debug!(node_id = %node_id, node_name = %node_name, strategy_output_handle_id = %strategy_output_handle_id, "setting strategy output handle");
        self.add_output_handle(strategy_output_handle_id, tx).await;

        let futures_order_configs = {
            let context = self.get_context();
            let context_guard = context.read().await;
            let futures_order_node_context = context_guard.as_any().downcast_ref::<FuturesOrderNodeContext>().unwrap();
            futures_order_node_context.backtest_config.futures_order_configs.clone()
        };
        // 为每一个订单添加出口
        for order_config in futures_order_configs.iter() {
            let created_output_handle_id = format!("{}_created_output_{}", node_id, order_config.order_config_id);
            let (created_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(created_output_handle_id, created_tx).await;

            match order_config.order_type {
                OrderType::Limit => {
                    let placed_output_handle_id = format!("{}_placed_output_{}", node_id, order_config.order_config_id);
                    let (placed_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
                    self.add_output_handle(placed_output_handle_id, placed_tx).await;

                }
                _ => {}
            }
            

            let partial_output_handle_id = format!("{}_partial_output_{}", node_id, order_config.order_config_id);
            let (partial_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(partial_output_handle_id, partial_tx).await;

            let filled_output_handle_id = format!("{}_filled_output_{}", node_id, order_config.order_config_id);
            let (filled_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(filled_output_handle_id, filled_tx).await;

            let canceled_output_handle_id = format!("{}_canceled_output_{}", node_id, order_config.order_config_id);
            let (canceled_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(canceled_output_handle_id, canceled_tx).await;

            let expired_output_handle_id = format!("{}_expired_output_{}", node_id, order_config.order_config_id);
            let (expired_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(expired_output_handle_id, expired_tx).await;

            let rejected_output_handle_id = format!("{}_rejected_output_{}", node_id, order_config.order_config_id);
            let (rejected_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(rejected_output_handle_id, rejected_tx).await;

            let error_output_handle_id = format!("{}_error_output_{}", node_id, order_config.order_config_id);
            let (error_tx, _) = broadcast::channel::<BacktestNodeEvent>(100);
            self.add_output_handle(error_output_handle_id, error_tx).await;
        }

        tracing::info!(node_id = %node_id, node_name = %node_name, "setting node handle complete");
    }

    async fn init(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("================={}====================", self.get_node_name().await);
        tracing::info!("{}: 开始初始化", self.get_node_name().await);
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await?;

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!("{:?}: 初始化完成", self.get_state_machine().await.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), BacktestStrategyNodeError> {
        tracing::info!("{}: 开始停止", self.get_node_id().await);
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await?;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await?;
        Ok(())
    }

    // 重写监听节点事件的方法
    async fn listen_node_events(&self) {
        let (input_handles, cancel_token, node_id) = {
            let state_guard = self.context.read().await;
            let input_handles = state_guard.get_all_input_handles().clone();
            let cancel_token = state_guard.get_cancel_token().clone();
            let node_id = state_guard.get_node_id().to_string();
            (input_handles, cancel_token, node_id)
        };

        if input_handles.is_empty() {
            tracing::warn!("{}: 没有消息接收器", node_id);
            return;
        }
        // 为每个接收器独立创建监听任务
        for input_handle in input_handles {
            let context = self.context.clone();
            let cancel_token = cancel_token.clone();
            let node_id = node_id.clone();
            let from_node_id = input_handle.from_node_id.clone();
            let input_handle_id = input_handle.input_handle_id.clone();
            
            // 为每个接收器创建独立的监听流
            let mut stream = BroadcastStream::new(input_handle.get_receiver());
            
            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        // 如果取消信号被触发，则中止任务
                        _ = cancel_token.cancelled() => {
                            tracing::info!("{} 节点接收器 {} 监听任务已中止", node_id, input_handle_id);
                            break;
                        }
                        // 接收消息
                        receive_result = stream.next() => {
                            match receive_result {
                                Some(Ok(node_event)) => {
                                    // 根据订单配置处理特定订单的事件
                                    let mut context_guard = context.write().await;
                                    if let Err(e) = context_guard.as_any_mut().downcast_mut::<FuturesOrderNodeContext>().unwrap().handle_node_event_for_specific_order(
                                        node_event, 
                                        &input_handle_id
                                    ).await {
                                        tracing::error!("节点{}处理特定订单事件错误: {}", node_id, e);
                                    }
                                }
                                Some(Err(e)) => {
                                    tracing::error!("节点{}接收器{}接收消息错误: {}", node_id, input_handle_id, e);
                                }
                                None => {
                                    tracing::warn!("节点{}接收器{}消息流已关闭", node_id, input_handle_id);
                                    break;
                                }
                            }
                        }
                    }
                }
            });
        }


    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), BacktestStrategyNodeError> {
        let node_id = self.get_node_id().await;

        // 获取状态管理器并执行转换
        let (transition_result, state_machine) = {
            let mut state_machine = self.get_state_machine().await;
            let transition_result = state_machine.transition(event)?;
            (transition_result, state_machine)
        };


        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(order_node_state_action) = action.as_any().downcast_ref::<OrderNodeStateAction>() {
                match order_node_state_action {
                    OrderNodeStateAction::LogTransition => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    OrderNodeStateAction::LogNodeState => {
                        let current_state = self.get_state_machine().await.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    OrderNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await;
                    }
                    OrderNodeStateAction::ListenAndHandleInnerEvents => {
                        tracing::info!("{}: 开始监听策略内部事件", node_id);
                        self.listen_strategy_inner_events().await;
                    }
                    OrderNodeStateAction::RegisterTask => {
                        tracing::info!("{}: 开始注册心跳任务", node_id);
                        let mut context_guard = self.context.write().await;
                        let order_node_context = context_guard.as_any_mut().downcast_mut::<FuturesOrderNodeContext>().unwrap();
                        order_node_context.monitor_unfilled_order().await;
                    }
                    OrderNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("{}: 开始监听节点消息", node_id);
                        self.listen_node_events().await;
                    }
                    OrderNodeStateAction::ListenAndHandleStrategyCommand => {
                        tracing::info!("{}: 开始监听策略命令", node_id);
                        self.listen_strategy_command().await;
                    }
                    
                    OrderNodeStateAction::ListenAndHandleVirtualTradingSystemEvent => {
                        tracing::info!("{}: 开始监听虚拟交易系统事件", node_id);
                        self.listen_virtual_trading_system_events().await;
                    }
                    OrderNodeStateAction::LogError(error) => {
                        tracing::error!("{}: 发生错误: {}", node_id, error);
                    }
                    OrderNodeStateAction::CancelAsyncTask => {
                        tracing::debug!(node_id = %node_id, "cancel async task");
                        self.cancel_task().await;
                    }
                }
                // 所有动作执行完毕后更新节点最新的状态
                {
                    self.context.write().await.set_state_machine(state_machine.clone_box());
                }
            }
        }
        Ok(())
    }
}


