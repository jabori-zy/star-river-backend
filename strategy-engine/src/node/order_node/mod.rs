
mod state_manager;

use types::market::{Exchange, KlineInterval};
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use event_center::market_event::MarketEvent;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::NodeMessageReceiver;
use crate::NodeType;
use tokio_util::sync::CancellationToken;
use crate::NodeOutputHandle;
use event_center::EventPublisher;
use crate::node::order_node::state_manager::{OrderNodeStateManager, OrderNodeStateAction};
use std::collections::HashMap;
use crate::NodeRunState;
use crate::node::NodeTrait;
use crate::NodeSender;
use crate::node::NodeStateTransitionEvent;
use event_center::response_event::ResponseEvent;
use event_center::response_event::ExchangeManagerResponse;
use futures::StreamExt;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use types::strategy::message::NodeMessage;
use std::time::Duration;
use types::strategy::message::Signal;
use types::order::OrderRequest;
use event_center::command_event::{CommandEvent, OrderEngineCommand, CreateOrderParams};

#[derive(Debug)]
pub struct OrderNodeState {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub request_id: Option<Uuid>,
    pub node_output_handle: HashMap<String, NodeOutputHandle>, // 节点的出口连接数 {handle_id: count}, 每个handle对应一个连接数
    pub event_publisher: EventPublisher, // 发送数据事件中心
    pub enable_event_publish: bool, // 是否启用事件发布
    pub cancel_token: CancellationToken,
    pub run_state_manager: OrderNodeStateManager,
    pub response_event_receiver: broadcast::Receiver<Event>, // 接收来自响应通道的数据
    pub node_receivers: Vec<NodeMessageReceiver>,
    pub order_request: OrderRequest, // 订单请求
}

#[derive(Debug, Clone)]
pub struct OrderNode {
    pub state: Arc<RwLock<OrderNodeState>>,
    pub from_node_id: Vec<String>, // 来自哪个节点的id
    pub node_type: NodeType,

}

impl OrderNode {
    pub fn new(
        strategy_id: i32,
        node_id: String,
        node_name: String,
        exchange: Exchange,
        symbol: String,
        order_request: OrderRequest,
        event_publisher: EventPublisher,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let cancel_token = CancellationToken::new();
        Self {
            node_type: NodeType::OrderNode,
            from_node_id: Vec::new(),
            state: Arc::new(RwLock::new(OrderNodeState {
                strategy_id: strategy_id.clone(),
                node_id: node_id.clone(),
                node_name: node_name.clone(),
                exchange,
                symbol,
                order_request,
                request_id: None,
                node_output_handle: HashMap::new(),
                event_publisher,
                enable_event_publish: false,
                cancel_token,
                run_state_manager: OrderNodeStateManager::new(NodeRunState::Created, node_id, node_name),
                response_event_receiver,
                node_receivers: Vec::new(),
            })),
        }
    }

    async fn update_run_state(state: Arc<RwLock<OrderNodeState>>, event: NodeStateTransitionEvent) -> Result<(), String> {
        let node_id = state.read().await.node_id.clone();

        // 获取状态管理器并执行转换
        let (transition_result, state_manager) = {
            let node_guard = state.read().await;  // 使用读锁获取当前状态
            let mut state_manager = node_guard.run_state_manager.clone();
            let transition_result = state_manager.transition(event)?;
            (transition_result, state_manager)
        };

        tracing::debug!("{}需要执行的动作: {:?}", node_id, transition_result.actions);

        // 执行转换后需要执行的动作
        for action in transition_result.actions.clone() {  // 克隆actions避免移动问题
            match action {
                OrderNodeStateAction::LogTransition => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.new_state);
                }
                OrderNodeStateAction::LogNodeState => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }
                OrderNodeStateAction::ListenAndHandleExternalEvents => {
                    tracing::info!("{}: 开始监听外部事件", node_id);
                    let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
                    Self::listen_external_events(state.clone(), internal_tx).await?;
                    Self::handle_external_events(state.clone(), internal_rx).await?;
                }
                OrderNodeStateAction::ListenAndHandleMessage => {
                    tracing::info!("{}: 开始监听节点消息", node_id);
                    Self::listen_message(state.clone()).await;
                }
                OrderNodeStateAction::LogError(error) => {
                    tracing::error!("{}: 发生错误: {}", node_id, error);
                }
            }
            // 所有动作执行完毕后更新节点最新的状态
            {
                let mut node_guard = state.write().await;
                node_guard.run_state_manager = state_manager.clone();
            }
        }
        Ok(())
    }

    // 监听外部事件
    async fn listen_external_events(
        state: Arc<RwLock<OrderNodeState>>,
        internal_tx: tokio::sync::mpsc::Sender<Event>,
    ) -> Result<(), String> {
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
                    Ok(event) = response_event_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });
        Ok(())

    }

    async fn handle_external_events(state: Arc<RwLock<OrderNodeState>>, mut internal_rx: tokio::sync::mpsc::Receiver<Event>) -> Result<(), String> {
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
                                Self::handle_response_event(state.clone(), response_event).await;
                            }
                            _ => {}
                        }
                    }
                    else => {
                        tracing::warn!("{} 节点事件通道已关闭", node_id);
                        break;
                    }
                }
            }
        });
        Ok(())
    }

    async fn handle_response_event(state: Arc<RwLock<OrderNodeState>>, response_event: ResponseEvent) {
        let request_id= {
            let state_guard = state.read().await;
            match state_guard.request_id {
                Some(id) => id,
                None => return,
            }
        };
        match response_event {
            ResponseEvent::ExchangeManager(ExchangeManagerResponse::RegisterExchangeSuccess(register_exchange_success_response)) => {
                if request_id == register_exchange_success_response.response_id {
                    let mut state_guard = state.write().await;
                    tracing::info!("{}: 交易所注册成功: {:?}", state_guard.node_id, register_exchange_success_response);
                    state_guard.request_id = None;
                }
            }
            _ => {}
        }
    }

    async fn listen_message(state: Arc<RwLock<OrderNodeState>>) {
        let event_publisher = state.read().await.event_publisher.clone();
        let cancel_token = state.read().await.cancel_token.clone();
        let node_id = state.read().await.node_id.clone();

        let state_guard = state.read().await;
        tracing::debug!("{}: receivers = {:?}", node_id, state_guard.node_receivers);

        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = state_guard.node_receivers.iter()
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
                                // tracing::debug!("节点{}接收到数据: {:?}", state.read().await.node_id, receive_message);
                                match receive_message {
                                    NodeMessage::Signal(signal_message) => {
                                        match signal_message.signal {
                                            // 如果信号为True，则执行下单
                                            Signal::True => {
                                                Self::create_order(state.clone()).await;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("节点{}接收消息错误: {}", state.read().await.node_id, e);
                            }
                            None => {
                                tracing::warn!("节点{}所有消息流已关闭", state.read().await.node_id);
                                break;
                            }
                        }
                    }
                }
            }
        });
        
    }

    async fn init_node_sender(self) -> Self {
        tracing::debug!("初始化订单节点handle");
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_output_handle = NodeOutputHandle {
            node_id: self.state.read().await.node_id.clone(),
            handle_id: "order_node_output".to_string(),
            sender: tx,
            connect_count: 0,
        };

        self.state.write().await.node_output_handle.insert("order_node_output".to_string(), node_output_handle);
        self
    }

    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    async fn create_order(state: Arc<RwLock<OrderNodeState>>) {
        let state_guard = state.read().await;
        let order_request = state_guard.order_request.clone();
        let create_order_params = CreateOrderParams {
            strategy_id: state_guard.strategy_id,
            node_id: state_guard.node_id.clone(),
            order_request,
            sender: state_guard.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: Uuid::new_v4(),
        };
        tracing::info!("{}: 发送创建订单命令: {:?}", state_guard.node_id, create_order_params);
        let command_event = CommandEvent::OrderEngine(OrderEngineCommand::CreateOrder(create_order_params));
        state_guard.event_publisher.publish(command_event.into()).expect("发送创建订单命令失败");
    }
    
}



#[async_trait]
impl NodeTrait for OrderNode {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    async fn get_node_id(&self) -> String {
        self.state.read().await.node_id.clone()
    }

    async fn get_node_name(&self) -> String {
        self.state.read().await.node_name.clone()
    }

    async fn get_node_sender(&self, handle_id: String) -> broadcast::Sender<NodeMessage> {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().sender.clone()
    }

    async fn get_default_node_sender(&self) -> broadcast::Sender<NodeMessage> {
        self.state.read().await.node_output_handle.get("order_node_output").unwrap().sender.clone()
    }

    async fn get_node_receivers(&self) -> Vec<NodeMessageReceiver> {
        self.state.read().await.node_receivers.clone()
    }

    async fn add_message_receiver(&mut self, receiver: NodeMessageReceiver) {
        self.state.write().await.node_receivers.push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.from_node_id.push(from_node_id);
    }

    async fn add_node_output_handle(&mut self, handle_id: String, sender: broadcast::Sender<NodeMessage>) {
        self.state.write().await.node_output_handle.insert(handle_id.clone(), NodeOutputHandle {
            node_id: self.state.read().await.node_id.clone(),
            handle_id: handle_id,
            sender: sender,
            connect_count: 0,
        });
    }

    async fn add_node_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.node_output_handle.get_mut(&handle_id).unwrap().connect_count += 1;
    }

    async fn enable_node_event_push(&mut self) {
        self.state.write().await.enable_event_publish = true;
        tracing::info!("{}: 节点事件推送已启用", self.state.read().await.node_name);
    }

    async fn disable_node_event_push(&mut self) {
        self.state.write().await.enable_event_publish = false;
        tracing::info!("{}: 节点事件推送已禁用", self.state.read().await.node_name);
    }

    async fn get_node_run_state(&self) -> NodeRunState {
        self.state.read().await.run_state_manager.current_state()
    }

    async fn init(&mut self) -> Result<(), String> {
        tracing::info!("================={}====================", self.state.read().await.node_name);
        tracing::info!("{}: 开始初始化", self.state.read().await.node_name);
        // 开始初始化 created -> Initialize
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::Initialize).await.unwrap();

        // 休眠500毫秒
        tokio::time::sleep(Duration::from_millis(500)).await;

        tracing::info!("{:?}: 初始化完成", self.state.read().await.run_state_manager.current_state());
        // 初始化完成 Initialize -> InitializeComplete
        Self::update_run_state(self.state.clone(), NodeStateTransitionEvent::InitializeComplete).await?;
        Ok(())
    }
    
}


