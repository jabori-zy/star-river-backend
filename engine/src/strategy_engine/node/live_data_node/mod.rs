mod live_data_node_state_machine;
mod live_data_node_context;

use types::market::{Exchange, KlineInterval};
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use event_center::Event;
use crate::*;
use tokio::sync::RwLock;
use std::sync::Arc;
use event_center::EventPublisher;
use super::node_types::NodeRunState;
use super::{NodeTrait,NodeType};
use super::NodeStateTransitionEvent;
use live_data_node_state_machine::{LiveDataNodeStateMachine, LiveDataNodeStateAction};
use super::node_context::{NodeContext,BaseNodeContext};
// 将需要共享的状态提取出来
use live_data_node_context::LiveDataNodeContext;

#[derive(Debug, Clone)]
pub struct LiveDataNode {
    pub context: Arc<RwLock<Box<dyn NodeContext>>>,
}

impl LiveDataNode {
    pub fn new(
        strategy_id: i64, 
        node_id: String, 
        node_name: String, 
        account_id: i32,
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        frequency: u32,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let base_context = BaseNodeContext::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            NodeType::LiveDataNode,
            event_publisher,
            vec![market_event_receiver, response_event_receiver],
            Box::new(LiveDataNodeStateMachine::new(node_id, node_name)),
        );
        Self {
            context: Arc::new(RwLock::new(Box::new(LiveDataNodeContext {
                base_context,
                account_id,
                exchange, 
                symbol, 
                interval, 
                frequency,
                is_subscribed: false,
                request_id: None
            }))), 
        }
    }
}

#[async_trait]
impl NodeTrait for LiveDataNode {

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }
    // 获取节点状态
    fn get_context(&self) -> Arc<RwLock<Box<dyn NodeContext>>> {
        self.context.clone()
    }


    
    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.context.read().await.get_node_name());
        tracing::info!("{}: 开始初始化", self.context.read().await.get_node_name());
        // 开始初始化 created -> Initialize
        self.update_run_state(NodeStateTransitionEvent::Initialize).await.unwrap();
        tracing::info!("{:?}: 初始化完成", self.context.read().await.get_state_machine().current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_run_state(NodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }

    async fn start(&mut self) -> Result<(), String> {
        let context = self.get_context();
        tracing::info!("{}: 开始启动", context.read().await.get_node_id());
        self.update_run_state(NodeStateTransitionEvent::Start).await.unwrap();

        // 检查是否应该订阅K线流，判断is_subscribed=true
        loop {
            let is_subscribed = {
                let state_guard = context.read().await;  // 使用读锁替代写锁
                if let Some(live_data_context) = state_guard.as_any().downcast_ref::<LiveDataNodeContext>() {
                    live_data_context.is_subscribed
                } else {
                    false
                }
            };  // 锁在这里释放

            if is_subscribed {
                break;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        self.update_run_state(NodeStateTransitionEvent::StartComplete).await?;
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.get_context();
        tracing::info!("{}: 开始停止", state.read().await.get_node_id());
        self.update_run_state(NodeStateTransitionEvent::Stop).await.unwrap();


        // 检查是否应该订阅K线流，判断is_subscribed=false
        loop {
            let is_subscribed = {
                let state_guard = state.read().await;  // 使用读锁替代写锁
                if let Some(live_data_context) = state_guard.as_any().downcast_ref::<LiveDataNodeContext>() {
                    live_data_context.is_subscribed
                } else {
                    false
                }
            };  // 锁在这里释放

            if !is_subscribed {
                break;
            }
            
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }

        self.update_run_state(NodeStateTransitionEvent::StopComplete).await?;
        self.cancel_task().await.unwrap();
        Ok(())
    }

    async fn update_run_state(&mut self, event: NodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = self.context.read().await.get_node_id().clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, run_state_manager) = {
            let mut run_state_manager = self.context.read().await.get_state_machine().clone_box();  // 使用读锁获取当前状态
            let transition_result = run_state_manager.transition(event)?;
            (transition_result, run_state_manager)
        };

        tracing::debug!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());
        
        
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {  // 克隆actions避免移动问题
            if let Some(live_data_node_state_action) = action.as_any().downcast_ref::<LiveDataNodeStateAction>() {
                match live_data_node_state_action {
                    LiveDataNodeStateAction::LogTransition => {
                        let current_state = self.context.read().await.get_state_machine().current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    LiveDataNodeStateAction::LogNodeState => {
                        let current_state = self.context.read().await.get_state_machine().current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    LiveDataNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    LiveDataNodeStateAction::RegisterExchange => {
                        tracing::info!("{}: 注册交易所", node_id);
                        let context = self.get_context();
                        let mut state_guard = context.write().await;
                        if let Some(live_data_context) = state_guard.as_any_mut().downcast_mut::<LiveDataNodeContext>() {
                            live_data_context.register_exchange().await?;
                        }
                    }
                    LiveDataNodeStateAction::SubscribeKline => {
                        let current_state = self.context.read().await.get_state_machine().current_state();
                        if current_state != NodeRunState::Starting {
                            tracing::warn!(
                                node_id = %node_id,
                                current_state = ?current_state,
                                "节点不在Starting状态, 不订阅K线流"
                            );
                        } else {
                            tracing::info!("{}: 订阅K线流", node_id);
                            let context = self.get_context();
                            let mut state_guard = context.write().await;
                            if let Some(live_data_context) = state_guard.as_any_mut().downcast_mut::<LiveDataNodeContext>() {
                                live_data_context.subscribe_kline_stream().await?;
                            }
                        }
                    }
                    LiveDataNodeStateAction::UnsubscribeKline => {
                        tracing::info!("{}: 取消订阅K线流", node_id);
                        let should_stop = {
                            let node_name = self.context.read().await.get_node_name().clone();
                            let current_state = self.context.read().await.get_state_machine().current_state();
                            if current_state != NodeRunState::Stopping {
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
                            if let Some(live_data_context) = state_guard.as_any_mut().downcast_mut::<LiveDataNodeContext>() {
                                live_data_context.unsubscribe_kline_stream().await?;
                            }
                        }
                    }
                    _ => {}
                }
            }
            // 动作执行完毕后更新节点最新的状态
            {
                self.context.write().await.set_state_machine(run_state_manager.clone_box());
            }
        }
        Ok(())
    }
}
