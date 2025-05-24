pub mod kline_node_state_machine;
pub mod kline_node_context;

use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use tokio::sync::RwLock;
use std::sync::Arc;
use event_center::EventPublisher;
use crate::strategy_engine::node::{LiveNodeTrait,NodeType};
use crate::strategy_engine::node::node_state_machine::*;
use kline_node_state_machine::{KlineNodeStateMachine, KlineNodeStateAction};
use crate::strategy_engine::node::node_context::{LiveNodeContextTrait,LiveBaseNodeContext};
use kline_node_context::{KlineNodeContext, KlineNodeLiveConfig};
use heartbeat::Heartbeat;
use tokio::sync::Mutex;
use event_center::{CommandPublisher, CommandReceiver, EventReceiver};
use types::strategy::node_command::NodeCommandSender;


#[derive(Debug, Clone)]
pub struct KlineNode {
    pub context: Arc<RwLock<Box<dyn LiveNodeContextTrait>>>,
}

impl KlineNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String, 
        live_config: KlineNodeLiveConfig,
        event_publisher: EventPublisher, 
        command_publisher: CommandPublisher,
        command_receiver: Arc<Mutex<CommandReceiver>>,
        market_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
        heartbeat: Arc<Mutex<Heartbeat>>,
        strategy_command_sender: NodeCommandSender,
    ) -> Self {
        let base_context = LiveBaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::KlineNode,
            event_publisher,
            vec![market_event_receiver, response_event_receiver],
            command_publisher,
            command_receiver,
            Box::new(KlineNodeStateMachine::new(node_id, node_name)),
            strategy_command_sender,
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(KlineNodeContext {
                base_context,
                stream_is_subscribed: Arc::new(RwLock::new(false)),
                exchange_is_registered: Arc::new(RwLock::new(false)),
                live_config,
                heartbeat,
            }))), 
        }
    }
}

#[async_trait]
impl LiveNodeTrait for KlineNode {

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn LiveNodeTrait> {
        Box::new(self.clone())
    }
    // 获取节点状态
    fn get_context(&self) -> Arc<RwLock<Box<dyn LiveNodeContextTrait>>> {
        self.context.clone()
    }


    
    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_node_state(LiveNodeStateTransitionEvent::Initialize).await.unwrap();
        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());

        // 检查交易所是否注册成功，并且K线流是否订阅成功
        loop {
            let is_registered_and_subscribed = {
                let state_guard = self.context.read().await;
                let kline_node_context = state_guard.as_any().downcast_ref::<KlineNodeContext>().unwrap();
                let is_registered = kline_node_context.exchange_is_registered.read().await.clone();
                let is_subscribed = kline_node_context.stream_is_subscribed.read().await.clone();
                is_registered && is_subscribed
            };
            if is_registered_and_subscribed {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        // 初始化完成 Initialize -> InitializeComplete
        self.update_node_state(LiveNodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        let context = self.get_context();
        tracing::info!("{}: 开始启动", context.read().await.get_node_id());
        // 开始启动 Starting -> Start
        // 启动前检查
        let is_registered_and_subscribed = {
            let state_guard = context.read().await;  // 使用读锁替代写锁
            if let Some(kline_node_context) = state_guard.as_any().downcast_ref::<KlineNodeContext>() {
                let is_subscribed = kline_node_context.stream_is_subscribed.read().await.clone();    
                let is_registered = kline_node_context.exchange_is_registered.read().await.clone();
                is_subscribed && is_registered
            } else {
                false
            }
        };  // 锁在这里释放

        if !is_registered_and_subscribed {
            tracing::warn!("{}: 交易所未注册或K线流未订阅, 不启动", context.read().await.get_node_id());
            return Err("交易所未注册或K线流未订阅".to_string());
        }

        self.update_node_state(LiveNodeStateTransitionEvent::Start).await.unwrap();
        self.update_node_state(LiveNodeStateTransitionEvent::StartComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.get_context();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_node_state(LiveNodeStateTransitionEvent::Stop).await.unwrap();


        // 检查是否应该订阅K线流，判断is_subscribed=false
        loop {
            let is_subscribed = {
                let state_guard = state.read().await;  // 使用读锁替代写锁
                if let Some(kline_node_context) = state_guard.as_any().downcast_ref::<KlineNodeContext>() {
                    let is_subscribed = kline_node_context.stream_is_subscribed.read().await.clone();
                    is_subscribed
                } else {
                    false
                }
            };  // 锁在这里释放

            if !is_subscribed {
                break;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        self.update_node_state(LiveNodeStateTransitionEvent::StopComplete).await?;
        self.cancel_task().await.unwrap();
        Ok(())
    }

    async fn update_node_state(&mut self, event: LiveNodeStateTransitionEvent) -> Result<(), String> {
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
                    KlineNodeStateAction::RegisterExchange => {
                        tracing::info!("{}: 注册交易所", node_id);
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) = state_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            let response = kline_node_context.register_exchange().await?;
                            if response.code() == 0 {
                                tracing::info!("{}注册交易所成功", node_id);
                                *kline_node_context.exchange_is_registered.write().await = true;
                            } else {
                                tracing::error!("{}注册交易所失败: {:?}", node_id, response);
                            }
                        }
                    }
                    KlineNodeStateAction::SubscribeKline => {
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) = state_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            let response = kline_node_context.subscribe_kline_stream().await?;
                            if response.code() == 0 {
                                tracing::info!("{}订阅K线流成功", node_id);
                                *kline_node_context.stream_is_subscribed.write().await = true;
                            } else {
                                tracing::error!("{}订阅K线流失败: {:?}", node_id, response);
                            }
                        }
                    }
                    KlineNodeStateAction::UnsubscribeKline => {
                        tracing::info!("{}: 取消订阅K线流", node_id);
                        let should_stop = {
                            let node_name = self.context.read().await.get_node_name().clone();
                            let current_state = self.context.read().await.get_state_machine().current_state();
                            if current_state != LiveNodeRunState::Stopping {
                                tracing::warn!(
                                    node_name = %node_name,
                                    current_state = ?current_state,
                                    "节点未运行, 不取消订阅K线流"
                                );
                                false
                            } else {
                                true
                            }
                        }; // 这里读锁被释放
                        if should_stop {
                            let context = self.get_context();
                            let mut state_guard = context.write().await;
                            if let Some(kline_node_context) = state_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                                let response = kline_node_context.unsubscribe_kline_stream().await?;
                                if response.code() == 0 {
                                    tracing::info!("{}取消订阅K线流成功", node_id);
                                    *kline_node_context.stream_is_subscribed.write().await = false;
                                } else {
                                    tracing::error!("{}取消订阅K线流失败: {:?}", node_id, response);
                                }
                            }
                        }
                    }
                    KlineNodeStateAction::RegisterTask => {
                        tracing::info!("{}: 注册任务", node_id);
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(kline_node_context) = state_guard.as_any_mut().downcast_mut::<KlineNodeContext>() {
                            kline_node_context.register_task().await;
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
