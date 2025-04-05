pub mod state_manager;

use types::indicator::Indicators;
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use async_trait::async_trait;
use futures::StreamExt;
use types::market::{Exchange, KlineInterval};
use event_center::{Event, EventPublisher};
use event_center::command_event::{CalculateIndicatorParams, CommandEvent, IndicatorEngineCommand};
use event_center::response_event::{ResponseEvent, IndicatorEngineResponse};
use event_center::strategy_event::StrategyEvent;
use utils::get_utc8_timestamp_millis;
use types::strategy::message::{IndicatorMessage, NodeMessage};
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::node::NodeTrait;
use crate::NodeMessageReceiver;
use crate::NodeOutputHandle;
use crate::NodeType;
use crate::NodeRunState;
use crate::node::NodeStateTransitionEvent;
use crate::node::indicator_node::state_manager::IndicatorNodeStateManager;
use crate::node::indicator_node::state_manager::IndicatorNodeStateAction;
use std::time::Duration;
use crate::node::base_node_state::BaseNodeState;
use crate::node::node_functions::NodeFunction;
use crate::node::state_machine::NodeStateMachine;

// 将需要共享的状态提取出来
#[derive(Debug)]
pub struct IndicatorNodeState {
    pub base_state: BaseNodeState<IndicatorNodeStateManager>,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: Indicators,
    pub current_batch_id: Option<String>,
    pub request_id: Option<Uuid>,
}


impl IndicatorNodeState {
    pub async fn handle_event(&mut self, event: Event) {
        match event {
            Event::Response(response_event) => {
                self.handle_response_event(response_event).await;
            }
            _ => {}
        }
    }

    async fn handle_response_event(&self, response_event: ResponseEvent) {
        match response_event {
            ResponseEvent::IndicatorEngine(indicator_engine_response) => {
                self.handle_indicator_engine_response(indicator_engine_response).await;
            }
            _ => {}
        }
    }

    async fn handle_indicator_engine_response(&self, indicator_engine_response: IndicatorEngineResponse) {
        match indicator_engine_response {
            IndicatorEngineResponse::CalculateIndicatorFinish(calculate_indicator_response) => {
                let (current_batch_id, request_id) = {
                    // 这里可能接收到别的节点的计算结果，而自己节点的current_batch_id和request_id为None，所以需要使用unwrap_or_default
                    let current_batch_id = self.current_batch_id.clone();
                    let request_id = self.request_id.clone();
                    if current_batch_id.is_none() || request_id.is_none() {
                        tracing::warn!("current_batch_id或request_id为None");
                        return;
                    }
                    (current_batch_id.unwrap_or_default(), request_id.unwrap_or_default())
                };
                let response_batch_id = calculate_indicator_response.batch_id;
                let response_id = calculate_indicator_response.response_id;
                // 如果请求id和批次id都匹配，则认为计算结果有效
                if current_batch_id == response_batch_id && request_id == response_id {
                    // 计算结果有效
                    let indicator = calculate_indicator_response.indicator;
                    let indicator_value = calculate_indicator_response.value;
                    // tracing::info!("节点{}计算指标完成: {:?}", state.read().await.node_id, indicator_value);
                    
                    let indicator_message = IndicatorMessage {
                        from_node_id: self.base_state.node_id.clone(),
                        from_node_name: self.base_state.node_name.clone(),
                        exchange: self.exchange.clone(),
                        symbol: self.symbol.clone(),
                        interval: self.interval.clone(),
                        indicator: indicator,
                        indicator_data: indicator_value,
                        batch_id: current_batch_id,
                        message_timestamp: get_utc8_timestamp_millis(),
                    };
                    // 获取handle的连接数
                    let default_handle_connect_count = self.base_state.output_handle.get("indicator_node_output").expect("指标节点默认的消息发送器不存在").connect_count;
                    // 如果连接数为0，则不发送数据
                    if default_handle_connect_count > 0 {
                        let default_output_handle = self.base_state.output_handle.get("indicator_node_output").expect("指标节点默认的消息发送器不存在");
                        match default_output_handle.sender.send(NodeMessage::Indicator(indicator_message.clone())) {
                            Ok(_) => {
                            // tracing::info!("节点{}发送指标数据: {:?} 发送成功, 接收者数量 = {}", state_guard.node_id, indicator_message, receiver_count);
                        }
                        Err(_) => {
                            tracing::error!("节点{}发送指标数据失败, 接收者数量 = {}", self.base_state.node_id, default_output_handle.connect_count);
                            }
                        }
                    } 
                    // 发送事件
                    if self.base_state.enable_event_publish {
                        let event = Event::Strategy(StrategyEvent::NodeMessage(NodeMessage::Indicator(indicator_message.clone())));
                        if let Err(_) = self.base_state.event_publisher.publish(event.into()) {
                            tracing::error!(
                                node_id = %self.base_state.node_id,
                                "指标节点发送数据失败"
                            );
                        }
                    }
                }
            }
        }
    }

    async fn handle_message(&mut self, message: NodeMessage) {
        match message {
            NodeMessage::KlineSeries(kline_series_message) => {
                // 向指标引擎发送计算请求
                let request_id = Uuid::new_v4();
                let batch_id = kline_series_message.batch_id;

                
                let calculate_indicator_params = CalculateIndicatorParams {
                    exchange: kline_series_message.exchange,
                    symbol: kline_series_message.symbol,
                    interval: kline_series_message.interval,
                    indicator: self.indicator.clone(),
                    kline_series: kline_series_message.kline_series,
                    sender: self.base_state.node_id.to_string(),
                    command_timestamp: get_utc8_timestamp_millis(),
                    request_id: request_id,
                    batch_id: batch_id.clone(),
                };

                self.current_batch_id = Some(batch_id);
                self.request_id = Some(request_id);

                let event = Event::Command(CommandEvent::IndicatorEngine(IndicatorEngineCommand::CalculateIndicator(calculate_indicator_params)));
                if let Err(e) = self.base_state.event_publisher.publish(event) {
                    tracing::error!("节点{}发送指标计算请求失败: {}", self.base_state.node_id, e);
                }
            }
            _ => {}
        }
    }
}




// 指标节点
#[derive(Debug)]
pub struct IndicatorNode {
    pub node_type: NodeType,
    pub from_node_id: Vec<String>,
    pub state: Arc<RwLock<IndicatorNodeState>>,
    
}

impl Clone for IndicatorNode {
    fn clone(&self) -> Self {
        Self {
            node_type: self.node_type.clone(), 
            from_node_id: self.from_node_id.clone(),
            state: self.state.clone(),
        }
    }
}




impl IndicatorNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        indicator: Indicators, 
        event_publisher: EventPublisher, 
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let base_state = BaseNodeState::new(
            strategy_id,
            node_id.clone(),
            node_name.clone(),
            event_publisher,
            vec![response_event_receiver],
            IndicatorNodeStateManager::new(NodeRunState::Created, node_id, node_name),
        );

        Self {
            node_type: NodeType::IndicatorNode,
            from_node_id: Vec::new(),
            state: Arc::new(RwLock::new(IndicatorNodeState {
                base_state,
                exchange,
                symbol,
                interval,
                indicator,
                current_batch_id: None,
                request_id: None,
            })),
            
        }
    }

    // 获取默认的handle
    pub async fn get_default_handle(state: &Arc<RwLock<IndicatorNodeState>>) -> NodeOutputHandle {
        let state = state.read().await;
        state.base_state.output_handle.get("indicator_node_output").unwrap().clone()
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    async fn init_node_sender(self) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_output_handle = NodeOutputHandle {
            node_id: self.state.read().await.base_state.node_id.clone(),
            handle_id: "indicator_node_output".to_string(),
            sender: tx,
            connect_count: 0,
        };

        self.state.write().await.base_state.output_handle.insert("indicator_node_output".to_string(), node_output_handle);
        self
    }

    async fn update_run_state(&self, event: NodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = self.state.read().await.base_state.node_id.clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, run_state_manager) = {
            let mut run_state_manager = self.state.read().await.base_state.run_state_manager.clone();  // 使用读锁获取当前状态
            let transition_result = run_state_manager.transition(event)?;
            (transition_result, run_state_manager)
        };

        tracing::info!("{}需要执行的动作: {:?}", node_id, transition_result.get_actions());
        // 执行转换后需要执行的动作
        for action in transition_result.get_actions() {
            if let Some(indicator_node_state_action) = action.as_any().downcast_ref::<IndicatorNodeStateAction>() {
                match indicator_node_state_action {
                    IndicatorNodeStateAction::LogTransition => {
                        let current_state = self.state.read().await.base_state.run_state_manager.current_state();
                        tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.get_new_state());
                    }
                    IndicatorNodeStateAction::LogNodeState => {
                        let current_state = self.state.read().await.base_state.run_state_manager.current_state();
                        tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                    }
                    IndicatorNodeStateAction::ListenAndHandleExternalEvents => {
                        tracing::info!("{}: 开始监听外部事件", node_id);
                        self.listen_external_events().await?;
                    }
                    IndicatorNodeStateAction::ListenAndHandleMessage => {
                        tracing::info!("{}: 开始监听节点传递的message", node_id);
                        self.listen_message().await?;
                    }
                    _ => {}
                }
                // 所有动作执行完毕后更新节点最新的状态
                {
                    let mut node_guard = self.state.write().await;
                    node_guard.base_state.run_state_manager = run_state_manager.clone();
                }
            }
        }
                    
        Ok(())
    }
    
    
}

#[async_trait]
impl NodeTrait for IndicatorNode {

    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    fn get_state(&self) -> Arc<RwLock<dyn Send + Sync + 'static>> {
        self.state.clone()
    }

    async fn get_output_handle(&self) -> HashMap<String, NodeOutputHandle> {
        self.state.read().await.base_state.output_handle.clone()
    }

    async fn get_node_name(&self) -> String {
        self.state.read().await.base_state.node_name.clone()
    }

    async fn get_node_id(&self) -> String {
        self.state.read().await.base_state.node_id.clone()
    }

    async fn get_message_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        self.state.read().await.base_state.output_handle.get(&handle_id).unwrap().sender.clone()
    }

    async fn get_default_message_sender(&self) -> broadcast::Sender<NodeMessage> {
        self.state.read().await.base_state.output_handle.get("indicator_node_output").unwrap().sender.clone()
    }

    async fn get_message_receivers(&self) -> Vec<NodeMessageReceiver> {
        self.state.read().await.base_state.message_receivers.clone()
    }

    async fn add_message_receiver(&mut self, receiver: NodeMessageReceiver) {
        self.state.write().await.base_state.message_receivers.push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.from_node_id.push(from_node_id);
    }

    async fn add_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<NodeMessage>) {
        self.state.write().await.base_state.output_handle.insert(handle_id.clone(), NodeOutputHandle {
            node_id: self.state.read().await.base_state.node_id.clone(),
            handle_id: handle_id.clone(),
            sender: sender.clone(),
            connect_count: 0,
        });
    }

    async fn add_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.base_state.output_handle.get_mut(&handle_id).unwrap().connect_count += 1;
    }



    async fn enable_node_event_push(&mut self) {
        self.state.write().await.base_state.enable_event_publish = true;
        tracing::info!("{}: 节点事件推送已启用", self.state.read().await.base_state.node_name);
    }

    async fn disable_node_event_push(&mut self) {
        self.state.write().await.base_state.enable_event_publish = false;
        tracing::info!("{}: 节点事件推送已禁用", self.state.read().await.base_state.node_name);
    }
    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.state.read().await.base_state.node_name);
        tracing::info!("{}: 开始初始化", self.state.read().await.base_state.node_name);
        // 开始初始化 created -> Initialize
        self.update_run_state(NodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.state.read().await.base_state.run_state_manager.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        self.update_run_state(NodeStateTransitionEvent::InitializeComplete).await?;

        Ok(())
    }
    async fn start(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始启动", state.read().await.base_state.node_id);
        self.update_run_state(NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        self.update_run_state(NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始停止", state.read().await.base_state.node_id);
        self.update_run_state(NodeStateTransitionEvent::Stop).await.unwrap();

        // 等待所有任务结束
        self.cancel_task().await;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        self.update_run_state(NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn get_node_run_state(&self) -> NodeRunState {
        self.state.read().await.base_state.run_state_manager.current_state()
    }

    async fn listen_external_events(&self) -> Result<(), String> {
        NodeFunction::listen_external_event(
            self.state.clone(),
            |state| &state.base_state.event_receivers,
            |state| &state.base_state.cancel_token,
            |state| &state.base_state.node_id,
            |event, state| {
                Box::pin(async move {
                    let mut state_guard = state.write().await;
                    state_guard.handle_event(event).await;
                })
            },
        ).await;
        Ok(())
    }

    async fn listen_message(&self) -> Result<(), String> {
        NodeFunction::listen_message(
            self.state.clone(),
            |state| &state.base_state.message_receivers,
            |state| &state.base_state.cancel_token,
            |state| &state.base_state.node_id,
            |message, state| {
                Box::pin(async move {
                    let mut state_guard = state.write().await;
                    state_guard.handle_message(message).await;
                })
            },
        ).await;
        Ok(())
    }

    async fn cancel_task(&self) -> Result<(), String> {
        NodeFunction::cancel_task(
            &self.state,
            |state| &state.base_state.cancel_token, 
            |state| &state.base_state.node_id, 
            |state| state.base_state.run_state_manager.current_state()
        ).await;
        Ok(())
    }
}