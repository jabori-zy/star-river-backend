pub mod kline_node_state_machine;
pub mod kline_node_context;
pub mod kline_node_type;

use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use tokio::sync::RwLock;
use std::sync::Arc;
use event_center::EventPublisher;
use crate::strategy_engine::node::{BacktestNodeTrait,NodeType};
use crate::strategy_engine::node::node_state_machine::*;
use kline_node_state_machine::{KlineNodeStateMachine, KlineNodeStateAction};
use crate::strategy_engine::node::node_context::{BacktestNodeContextTrait,BacktestBaseNodeContext};
use kline_node_context::{KlineNodeContext};
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use kline_node_type::KlineNodeBacktestConfig;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::{StrategyInnerEventReceiver};

#[derive(Debug, Clone)]
pub struct KlineNode {
    pub context: Arc<RwLock<Box<dyn BacktestNodeContextTrait>>>,
}

impl KlineNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String, 
        backtest_config: KlineNodeBacktestConfig,
        event_publisher: EventPublisher, 
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        market_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Self {
        let base_context = BacktestBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::KlineNode,
            event_publisher,
            vec![market_event_receiver, response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(KlineNodeStateMachine::new(node_id, node_name, backtest_config.data_source.clone())),
            strategy_command_sender,
            strategy_inner_event_receiver,
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(KlineNodeContext {
                base_context,
                data_is_loaded: Arc::new(RwLock::new(false)),
                exchange_is_registered: Arc::new(RwLock::new(false)),
                backtest_config,
                heartbeat,
                kline_cache_index: Arc::new(RwLock::new(0)),
            }))), 
        }
    }
}

#[async_trait]
impl BacktestNodeTrait for KlineNode {

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn BacktestNodeTrait> {
        Box::new(self.clone())
    }
    // 获取节点状态
    fn get_context(&self) -> Arc<RwLock<Box<dyn BacktestNodeContextTrait>>> {
        self.context.clone()
    }


    
    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(BacktestNodeStateTransitionEvent::Initialize).await.unwrap();
        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());

        // 检查交易所是否注册成功，并且K线流是否订阅成功
        loop {
            let is_registered_and_data_loaded = {
                let state_guard = self.context.read().await;
                let kline_node_context = state_guard.as_any().downcast_ref::<KlineNodeContext>().unwrap();
                let is_registered = kline_node_context.exchange_is_registered.read().await.clone();
                let is_data_loaded = kline_node_context.data_is_loaded.read().await.clone();
                is_registered && is_data_loaded
            };
            if is_registered_and_data_loaded {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(BacktestNodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.get_context();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(BacktestNodeStateTransitionEvent::Stop).await.unwrap();


        // 检查是否应该订阅K线流，判断is_data_loaded=false
        loop {
            let is_data_loaded = {
                let state_guard = state.read().await;  // 使用读锁替代写锁
                if let Some(kline_node_context) = state_guard.as_any().downcast_ref::<KlineNodeContext>() {
                    let is_data_loaded = kline_node_context.data_is_loaded.read().await.clone();
                    is_data_loaded
                } else {
                    false
                }
            };  // 锁在这里释放

            if !is_data_loaded {
                break;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        self.update_node_state(BacktestNodeStateTransitionEvent::StopComplete).await?;
        self.cancel_task().await.unwrap();
        Ok(())
    }

    async fn update_node_state(&mut self, event: BacktestNodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = self.context.read().await.get_node_id().clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, state_machine) = {
            let mut state_machine = self.context.read().await.get_state_machine().clone_box();  // 使用读锁获取当前状态
            let transition_result = state_machine.transition(event)?;
            (transition_result, state_machine)
        };

        tracing::debug!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());
        
        
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(kline_node_state_action) = action.as_any().downcast_ref::<KlineNodeStateAction>() {
                match kline_node_state_action {
                    KlineNodeStateAction::LogTransition => {
                        let current_state = self.context.read().await.get_state_machine().current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    KlineNodeStateAction::LogNodeState => {
                        let current_state = self.context.read().await.get_state_machine().current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    KlineNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    KlineNodeStateAction::ListenAndHandleNodeEvents => {
                        tracing::info!("{}: 开始监听节点消息", node_id);
                        self.listen_node_events().await?;
                    }
                    KlineNodeStateAction::ListenAndHandleInnerEvents => {
                        tracing::info!("{}: 开始监听策略内部事件", node_id);
                        self.listen_strategy_inner_events().await?;
                    }
                    KlineNodeStateAction::RegisterExchange => {
                        tracing::info!("{}: 注册交易所", node_id);
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) = state_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            
                            let response = kline_node_context.register_exchange().await?;
                            if response.code() == 0 {
                                *kline_node_context.exchange_is_registered.write().await = true;
                                tracing::info!("{}注册交易所成功", node_id);   
                            } else {
                                tracing::error!("{}注册交易所失败: {:?}", node_id, response);
                            }
                        }
                    }
                    KlineNodeStateAction::LoadHistoryFromExchange => {
                        tracing::info!("{}: 从交易所加载K线历史", node_id);
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) = state_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            let response = kline_node_context.load_kline_history_from_exchange().await?;
                            if response.code() == 0 {
                                // 加载K线历史成功后，设置data_is_loaded=true
                                *kline_node_context.data_is_loaded.write().await = true;
                                tracing::info!("{}从交易所加载K线历史成功", node_id);
                            } else {
                                tracing::error!("{}从交易所加载K线历史失败: {:?}", node_id, response);
                            }
                        }
                    }

                    _ => {}
                }
            }
            // 动作执行完毕后更新节点最新的状态
            {
                self.context.write().await.set_state_machine(state_machine.clone_box());
            }
        }
        Ok(())
    }
}
