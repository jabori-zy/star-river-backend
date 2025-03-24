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
use crate::NodeSender;
use crate::NodeReceiver;
use crate::NodeOutputHandle;
use crate::NodeType;
use crate::NodeRunState;
use tokio_util::sync::CancellationToken;
use crate::node::NodeStateTransitionEvent;
use crate::node::indicator_node::state_manager::IndicatorNodeStateManager;
use crate::node::indicator_node::state_manager::IndicatorNodeStateAction;
use std::time::Duration;

// 将需要共享的状态提取出来
#[derive(Debug)]
pub struct IndicatorNodeState { 
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub indicator: Indicators,
    pub current_batch_id: Option<String>,
    pub request_id: Option<Uuid>,
    // pub node_sender: NodeSender,
    pub node_output_handle: HashMap<String, NodeSender>, // 节点的出口 {handle_id: sender}, 每个handle对应一个sender
    pub node_output_handle1: HashMap<String, NodeOutputHandle>, // 节点的出口连接数 {handle_id: count}, 每个handle对应一个连接数
    pub event_publisher: EventPublisher,
    pub enable_event_publish: bool, // 是否启用事件发布
    pub cancel_token: CancellationToken,
    pub run_state_manager: IndicatorNodeStateManager,
    pub node_receivers: Vec<NodeReceiver>,
    pub response_event_receiver: broadcast::Receiver<Event>,
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
        exchange: Exchange, symbol: String, 
        interval: KlineInterval, 
        indicator: Indicators, 
        event_publisher: EventPublisher, 
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {

        Self { 
            node_type: NodeType::IndicatorNode,
            from_node_id: Vec::new(),
            state: Arc::new(RwLock::new(IndicatorNodeState {
                strategy_id,
                node_id: node_id.clone(),
                node_name: node_name.clone(),
                exchange,
                symbol,
                interval,
                indicator,
                current_batch_id: None,
                request_id: None,
                node_receivers: Vec::new(),
                node_output_handle: HashMap::new(),
                node_output_handle1: HashMap::new(),
                event_publisher,
                enable_event_publish: false,
                cancel_token: CancellationToken::new(),
                run_state_manager: IndicatorNodeStateManager::new(NodeRunState::Created, node_id, node_name),
                response_event_receiver,
            })),
            
        }
    }

    // 监听事件
    async fn listen_external_events(state: Arc<RwLock<IndicatorNodeState>>, internal_tx: mpsc::Sender<Event>,) -> Result<(), String> {
        // 接收指标引擎返回的计算结果，并发送给下一个节点
        // 指标引擎响应的接收器
        let state = state.read().await;
        let mut response_event_receiver = state.response_event_receiver.resubscribe();
        let cancel_token = state.cancel_token.clone();
        let node_id = state.node_id.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点监听外部事件进程已中止", node_id);
                        break;
                    }
                    Ok(response_event) = response_event_receiver.recv() => {
                        let _ = internal_tx.send(response_event).await;
                    }
                }
            }
        });
        
        Ok(())
    }

    // 处理接收到的事件
    async fn handle_external_events(state: Arc<RwLock<IndicatorNodeState>>, mut internal_rx: mpsc::Receiver<Event>) {
        let cancel_token = state.read().await.cancel_token.clone();
        let node_id = state.read().await.node_id.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点处理外部事件任务已中止", node_id);
                        break;
                    }
                    Some(event) = internal_rx.recv() => {
                        match event {
                            Event::Response(response_event) => {
                                IndicatorNode::handle_response_event(state.clone(), response_event).await;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });
    }

    // 处理接收到的事件
    async fn handle_response_event(state: Arc<RwLock<IndicatorNodeState>>, response_event: ResponseEvent) {
        match response_event {
            ResponseEvent::IndicatorEngine(indicator_engine_response) => {
                IndicatorNode::handle_indicator_engine_response(state, indicator_engine_response).await;
            }
            _ => {}
        }
    }
    
    // 处理指标引擎返回的计算结果
    async fn handle_indicator_engine_response(state: Arc<RwLock<IndicatorNodeState>>, indicator_engine_response: IndicatorEngineResponse) {
        match indicator_engine_response {
            IndicatorEngineResponse::CalculateIndicatorFinish(calculate_indicator_response) => {
                let (current_batch_id, request_id) = {
                    let state_guard = state.read().await;
                    // 这里可能接收到别的节点的计算结果，而自己节点的current_batch_id和request_id为None，所以需要使用unwrap_or_default
                    let current_batch_id = state_guard.current_batch_id.clone();
                    let request_id = state_guard.request_id.clone();
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
                    let state_guard = state.read().await;
                    
                    let indicator_message = IndicatorMessage {
                        from_node_id: state_guard.node_id.clone(),
                        from_node_name: state_guard.node_name.clone(),
                        exchange: state_guard.exchange.clone(),
                        symbol: state_guard.symbol.clone(),
                        interval: state_guard.interval.clone(),
                        indicator: indicator,
                        indicator_data: indicator_value,
                        batch_id: current_batch_id,
                        message_timestamp: get_utc8_timestamp_millis(),
                    };
                    // 获取handle的连接数
                    let default_handle_connect_count = state_guard.node_output_handle1.get("indicator_node_output").expect("指标节点默认的消息发送器不存在").connect_count;
                    // 如果连接数为0，则不发送数据
                    if default_handle_connect_count > 0 {
                        let default_node_sender = state_guard.node_output_handle.get("indicator_node_output").expect("指标节点默认的消息发送器不存在");
                        match default_node_sender.send(NodeMessage::Indicator(indicator_message.clone())) {
                            Ok(_) => {
                            // tracing::info!("节点{}发送指标数据: {:?} 发送成功, 接收者数量 = {}", state_guard.node_id, indicator_message, receiver_count);
                        }
                        Err(_) => {
                            tracing::error!("节点{}发送指标数据失败, 接收者数量 = {}", state_guard.node_id, default_node_sender.receiver_count());
                            }
                        }
                    } 
                    // 发送事件
                    if state_guard.enable_event_publish {
                        let event = Event::Strategy(StrategyEvent::NodeMessage(NodeMessage::Indicator(indicator_message.clone())));
                        if let Err(_) = state_guard.event_publisher.publish(event.into()) {
                            tracing::error!(
                                node_id = %state_guard.node_id,
                                "指标节点发送数据失败"
                            );
                        }
                    }
                }
            }
        }
    }

    // 监听节点传递过来的message
    async fn listen_message(state: Arc<RwLock<IndicatorNodeState>>) {
        let event_publisher = state.read().await.event_publisher.clone();
        let cancel_token = state.read().await.cancel_token.clone();
        let node_id = state.read().await.node_id.clone();
        
        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = state.read().await.node_receivers.iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();
        let mut combined_stream = select_all(streams);

        let state = state.clone();
        // 指标节点接收数据源节点的数据
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 如果取消信号被触发，则中止任务
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点消息监听任务已中止", node_id);
                        break;
                    }
                    // 接收消息
                    receive_result = combined_stream.next() => {
                        match receive_result {
                            Some(Ok(receive_message)) => {
                                tracing::info!("节点{}接收到数据: {:?}", state.read().await.node_id, receive_message);
                                match receive_message {
                                    NodeMessage::KlineSeries(kline_series_message) => {
                                        // 向指标引擎发送计算请求
                                        let request_id = Uuid::new_v4();
                                        let batch_id = kline_series_message.batch_id;

                                        let (node_id, indicator) = {
                                            let state_guard = state.read().await;
                                            (state_guard.node_id.clone(), state_guard.indicator.clone())
                                        };
                                        
                                        let calculate_indicator_params = CalculateIndicatorParams {
                                            exchange: kline_series_message.exchange,
                                            symbol: kline_series_message.symbol,
                                            interval: kline_series_message.interval,
                                            indicator: indicator,
                                            kline_series: kline_series_message.kline_series,
                                            sender: node_id.to_string(),
                                            command_timestamp: get_utc8_timestamp_millis(),
                                            request_id: request_id,
                                            batch_id: batch_id.clone(),
                                        };
                                        // 设置state
                                        {
                                            let mut state_guard = state.write().await;
                                            state_guard.current_batch_id = Some(batch_id);
                                            state_guard.request_id = Some(request_id);
                                        }

                                        let event = Event::Command(CommandEvent::IndicatorEngine(IndicatorEngineCommand::CalculateIndicator(calculate_indicator_params)));
                                        if let Err(e) = event_publisher.publish(event) {
                                            tracing::error!("节点{}发送指标计算请求失败: {}", node_id, e);
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("节点{}接收消息错误: {}", state.read().await.node_id, e);
                            }
                            None => {
                                tracing::info!("节点{}所有消息流已关闭", state.read().await.node_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
    }

    // 获取默认的handle
    pub async fn get_default_handle(state: &Arc<RwLock<IndicatorNodeState>>) -> NodeSender {
        let state = state.read().await;
        state.node_output_handle.get("indicator_node_output").unwrap().clone()
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    async fn init_node_sender(self) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let indicator_node_sender = NodeSender::new(self.state.read().await.node_id.clone(), "indicator_node_output".to_string(), tx);
        self.state.write().await.node_output_handle.insert("indicator_node_output".to_string(), indicator_node_sender.clone());
        self.state.write().await.node_output_handle1.insert("indicator_node_output".to_string(), NodeOutputHandle {
            handle_id: "indicator_node_output".to_string(),
            sender: indicator_node_sender.clone(),
            connect_count: 0,
        });
        self
    }

    async fn update_run_state(state: Arc<RwLock<IndicatorNodeState>>, event: NodeStateTransitionEvent) -> Result<(), String> {
        // 提前获取所有需要的数据，避免在循环中持有引用
        let node_id = state.read().await.node_id.clone();
        
        // 获取状态管理器并执行转换
        let (transition_result, state_manager) = {
            let node_guard = state.read().await;  // 使用读锁获取当前状态
            let mut state_manager = node_guard.run_state_manager.clone();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        tracing::info!("{}需要执行的动作: {:?}", node_id, transition_result.actions);
        // 执行转换后需要执行的动作
        for action in transition_result.actions.clone() {  // 克隆actions避免移动问题
            match action {
                IndicatorNodeStateAction::LogTransition => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.new_state);
                }
                IndicatorNodeStateAction ::LogNodeState => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }
                IndicatorNodeStateAction::ListenAndHandleExternalEvents => {
                    tracing::info!("{}: 开始监听外部事件", node_id);
                    let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
                    Self::listen_external_events(state.clone(), internal_tx).await.unwrap();
                    Self::handle_external_events(state.clone(), internal_rx).await;
                }
                IndicatorNodeStateAction::ListenAndHandleMessage => {
                    tracing::info!("{}: 开始监听节点传递的message", node_id);
                    Self::listen_message(state.clone()).await;
                }
                _ => {}
                // 所有动作执行完毕后更新节点最新的状态
                
            }
            // 所有动作执行完毕后更新节点最新的状态
            {
                let mut node_guard = state.write().await;
                node_guard.run_state_manager = state_manager.clone();
            }
        }
                    
        Ok(())
    }

    
    async fn cancel_task(state: Arc<RwLock<IndicatorNodeState>>) {
        let state_guard = state.read().await;
        state_guard.cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", state_guard.node_id, state_guard.run_state_manager.current_state());
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


    async fn get_node_name(&self) -> String {
        self.state.read().await.node_name.clone()
    }

    async fn get_node_id(&self) -> String {
        self.state.read().await.node_id.clone()
    }

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.node_output_handle.get("indicator_node_output").unwrap().clone()
    }

    async fn add_message_receiver(&mut self, receiver: NodeReceiver) {
        self.state.write().await.node_receivers.push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.from_node_id.push(from_node_id);
    }

    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender) {
        self.state.write().await.node_output_handle.insert(handle_id.clone(), sender.clone());
        self.state.write().await.node_output_handle1.insert(handle_id.clone(), NodeOutputHandle {
            handle_id: handle_id.clone(),
            sender: sender.clone(),
            connect_count: 0,
        });
    }

    async fn add_node_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.node_output_handle1.get_mut(&handle_id).unwrap().connect_count += 1;
    }



    async fn enable_node_event_publish(&mut self) {
        self.state.write().await.enable_event_publish = true;
    }

    async fn disable_node_event_publish(&mut self) {
        self.state.write().await.enable_event_publish = false;
    }
    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.state.read().await.node_name);
        tracing::info!("{}: 开始初始化", self.state.read().await.node_name);
        // 开始初始化 created -> Initialize
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::Initialize).await.unwrap();

        tracing::info!("{:?}: 初始化完成", self.state.read().await.run_state_manager.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::InitializeComplete).await?;

        Ok(())
    }
    async fn start(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始启动", state.read().await.node_id);
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Start).await.unwrap();
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为running状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StartComplete).await.unwrap();
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始停止", state.read().await.node_id);
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Stop).await.unwrap();

        // 等待所有任务结束
        Self::cancel_task(state.clone()).await;
        // 休眠500毫秒
        tokio::time::sleep(Duration::from_secs(1)).await;
        // 切换为stopped状态
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::StopComplete).await.unwrap();
        Ok(())
    }

    async fn get_node_run_state(&self) -> NodeRunState {
        self.state.read().await.run_state_manager.current_state()
    }
}