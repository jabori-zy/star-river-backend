mod state_manager;

use event_center::command_event::{SubscribeKlineStreamParams, MarketDataEngineCommand, CommandEvent, UnsubscribeKlineStreamParams};
use types::market::{Exchange, KlineInterval};
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use event_center::market_event::MarketEvent;
use crate::*;
use types::strategy::message::{KlineSeriesMessage, NodeMessage};
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;
use event_center::EventPublisher;
use tokio::sync::mpsc;
use event_center::command_event::{RegisterExchangeParams, ExchangeManagerCommand};
use event_center::response_event::{MarketDataEngineResponse, ResponseEvent, ExchangeManagerResponse};
use event_center::strategy_event::StrategyEvent;
use std::collections::HashMap;
use crate::NodeOutputHandle;
use crate::node::NodeRunState;
use crate::node::NodeTrait;
use tokio_util::sync::CancellationToken;
use crate::node::NodeStateTransitionEvent;
use crate::node::live_data_node::state_manager::{LiveDataNodeStateManager, LiveDataNodeStateAction};

// 将需要共享的状态提取出来
#[derive(Debug)]
pub struct LiveDataNodeState {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub frequency: u32,
    pub request_id: Option<Uuid>,
    // pub node_run_state: NodeRunState,
    pub output_handle: HashMap<String, NodeOutputHandle>, // 节点的出口连接数 {handle_id: count}, 每个handle对应一个连接数
    pub event_publisher: EventPublisher, // 发送数据事件中心
    pub enable_event_publish: bool, // 是否启用事件发布
    pub cancel_token: CancellationToken,
    pub run_state_manager: LiveDataNodeStateManager,
    pub market_event_receiver: broadcast::Receiver<Event>, // 接收来自市场通道的数据
    pub response_event_receiver: broadcast::Receiver<Event>, // 接收来自响应通道的数据
}

#[derive(Debug)]
pub struct LiveDataNode {
    pub state: Arc<RwLock<LiveDataNodeState>>,
    pub message_receivers: Vec<NodeReceiver>, // 接收来自其他节点的数据
    pub from_node_id: Vec<String>, // 来自哪个节点的id
    pub node_type: NodeType,
}

impl Clone for LiveDataNode {
    fn clone(&self) -> Self {
        LiveDataNode { 
            node_type: self.node_type.clone(),
            from_node_id: self.from_node_id.clone(),
            message_receivers: self.message_receivers.clone(), 
            state: self.state.clone(),
        }
    }
}

impl LiveDataNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        node_name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        frequency: u32,
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
    ) -> Self {
        let cancel_token = CancellationToken::new();
        Self { 
            node_type: NodeType::DataSourceNode, 
            message_receivers: Vec::new(),
            from_node_id: Vec::new(),
            state: Arc::new(RwLock::new(LiveDataNodeState {
                strategy_id,
                node_id: node_id.clone(), 
                node_name: node_name.clone(), 
                exchange, 
                symbol, 
                interval, 
                frequency,
                request_id: None,
                output_handle: HashMap::new(),
                event_publisher,
                enable_event_publish: false,
                cancel_token,
                market_event_receiver,
                response_event_receiver,
                run_state_manager: LiveDataNodeStateManager::new(
                    NodeRunState::Created,
                    node_id,
                    node_name
                ),
            }
        )), 
        }
    }

    async fn update_run_state(state: Arc<RwLock<LiveDataNodeState>>, event: NodeStateTransitionEvent) -> Result<(), String> {
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
                LiveDataNodeStateAction::LogTransition => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 状态转换: {:?} -> {:?}", node_id, current_state, transition_result.new_state);
                }
                LiveDataNodeStateAction::LogNodeState => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    tracing::info!("{}: 当前状态: {:?}", node_id, current_state);
                }
                LiveDataNodeStateAction::ListenAndHandleExternalEvents => {
                    tracing::info!("{}: 开始监听外部事件", node_id);
                    let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
                    Self::listen_external_events(state.clone(), internal_tx).await?;
                    Self::handle_external_events(state.clone(), internal_rx).await;
                }
                LiveDataNodeStateAction::RegisterExchange => {
                    tracing::info!("{}: 注册交易所", node_id);
                    Self::register_exchange(state.clone()).await?;
                }
                LiveDataNodeStateAction::SubscribeKline => {
                    let current_state = state.read().await.run_state_manager.current_state();
                    if current_state != NodeRunState::Starting {
                        tracing::warn!(
                            node_id = %node_id,
                            current_state = ?current_state,
                            "节点不在Starting状态, 不订阅K线流"
                        );
                    } else {
                        tracing::info!("{}: 订阅K线流", node_id);
                        Self::subscribe_kline_stream(state.clone()).await?;
                    }
                }
                LiveDataNodeStateAction::UnsubscribeKline => {
                    tracing::info!("{}: 取消订阅K线流", node_id);
                    let should_stop = {
                        let state_guard = state.read().await;
                        let node_name = state_guard.node_name.clone();
                        let current_state = state_guard.run_state_manager.current_state();
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
                        Self::unsubscribe_kline_stream(state.clone()).await?;
                    }
                }
                _ => {}
            }
            // 所有动作执行完毕后更新节点最新的状态
            {
                let mut node_guard = state.write().await;
                node_guard.run_state_manager = state_manager.clone();
            }

            
            
        }
        

        Ok(())
    }

    async fn register_exchange(state: Arc<RwLock<LiveDataNodeState>>) -> Result<(), String> {
        let mut state_guard = state.write().await;
        let request_id = Uuid::new_v4();
        let register_param = RegisterExchangeParams {
            exchange: state_guard.exchange.clone(),
            sender: state_guard.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        state_guard.request_id = Some(request_id);

        let command_event = CommandEvent::ExchangeManager(ExchangeManagerCommand::RegisterExchange(register_param));
        tracing::info!("{}注册交易所: {:?}", state_guard.node_id, command_event);
        if let Err(e) = state_guard.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %state_guard.node_id,
                error = ?e,
                "数据源节点发送注册交易所失败"
            );
        }
        Ok(())
        
        
    }

    async fn subscribe_kline_stream(state: Arc<RwLock<LiveDataNodeState>>) -> Result<(), String> {
        let mut state_guard = state.write().await;
        let request_id = Uuid::new_v4();
        let params = SubscribeKlineStreamParams {
            strategy_id: state_guard.strategy_id.clone(),
            node_id: state_guard.node_id.clone(),
            exchange: state_guard.exchange.clone(),
            symbol: state_guard.symbol.clone(),
            interval: state_guard.interval.clone(),
            frequency: state_guard.frequency.clone(),
            sender: state_guard.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        state_guard.request_id = Some(request_id);

        let command_event = CommandEvent::MarketDataEngine(MarketDataEngineCommand::SubscribeKlineStream(params));
        tracing::info!("{}订阅k线流: {:?}", state_guard.node_id, command_event);
        if let Err(e) = state_guard.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %state_guard.node_id,
                error = ?e,
                "数据源节点发送数据失败"
            );
        }
        Ok(())
    }

    async fn unsubscribe_kline_stream(state: Arc<RwLock<LiveDataNodeState>>) -> Result<(), String> {
        let mut state_guard = state.write().await;
        let request_id = Uuid::new_v4();
        let params = UnsubscribeKlineStreamParams {
            strategy_id: state_guard.strategy_id.clone(),
            node_id: state_guard.node_id.clone(),
            exchange: state_guard.exchange.clone(),
            symbol: state_guard.symbol.clone(),
            interval: state_guard.interval.clone(),
            frequency: state_guard.frequency.clone(),
            sender: state_guard.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        // 设置请求id
        state_guard.request_id = Some(request_id);

        let command_event = CommandEvent::MarketDataEngine(MarketDataEngineCommand::UnsubscribeKlineStream(params));
        if let Err(_) = state_guard.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %state_guard.node_id,
                "数据源节点发送数据失败"
            );
        }   
        Ok(())
    }


    // 监听外部事件
    async fn listen_external_events(
        state: Arc<RwLock<LiveDataNodeState>>,
        internal_tx: mpsc::Sender<Event>,
    ) -> Result<(), String> {
        let state = state.read().await;
        let mut response_event_receiver = state.response_event_receiver.resubscribe();
        let mut market_event_receiver = state.market_event_receiver.resubscribe();
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
                    Ok(event) = market_event_receiver.recv() => {
                        let _ = internal_tx.send(event).await;
                    }
                }
            }
        });
        Ok(())

    }
    // 处理接收到的外部事件
    async fn handle_external_events(state: Arc<RwLock<LiveDataNodeState>>, mut internal_rx: mpsc::Receiver<Event>) {
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
                            Event::Market(market_event) => {
                                Self::handle_market_event(state.clone(), market_event).await;
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
    }
    
    async fn handle_market_event(state: Arc<RwLock<LiveDataNodeState>>, market_event: MarketEvent) {
        // 先获取读锁，检查状态
        let state_guard = state.read().await;

        if state_guard.run_state_manager.current_state() != NodeRunState::Running {
            tracing::warn!("{}: 节点状态不是Running, 不处理行情数据", state_guard.node_id);
            return;
        }

        // 处理市场事件
        match market_event {
            MarketEvent::KlineSeriesUpdate(kline_series_update) => {
                // 只获取当前节点支持的数据
                let exchange = state_guard.exchange.clone();
                let symbol = state_guard.symbol.clone();
                let interval = state_guard.interval.clone();
                if exchange != kline_series_update.exchange || symbol != kline_series_update.symbol || interval != kline_series_update.interval {
                    return;
                }
                // 这里不需要再获取锁，因为我们只需要读取数据
                let kline_series_message = KlineSeriesMessage {
                    from_node_id: state_guard.node_id.clone(),
                    from_node_name: state_guard.node_name.clone(),
                    exchange: kline_series_update.exchange,
                    symbol: kline_series_update.symbol,
                    interval: kline_series_update.interval,
                    kline_series: kline_series_update.kline_series.clone(),
                    batch_id: kline_series_update.batch_id.clone(),
                    message_timestamp: get_utc8_timestamp_millis(),
                };
                
                let message = NodeMessage::KlineSeries(kline_series_message);
                // tracing::info!("{}: 发送数据: {:?}", state_guard.node_id, message);
                // 获取handle的连接数
                let default_handle_connect_count = state_guard.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在").sender.receiver_count();
                // 如果连接数为0，则不发送数据
                if default_handle_connect_count > 0 {
                    let default_node_sender = state_guard.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在");
                    tracing::info!("{}: 发送数据: {:?}", state_guard.node_id, message);
                    match default_node_sender.sender.send(message.clone()) {
                        Ok(_) => (),
                        Err(e) => tracing::error!(
                            node_id = %state_guard.node_id,
                            error = ?e,
                            receiver_count = default_node_sender.sender.receiver_count(),
                                "数据源节点发送数据失败"
                            ),
                        }
                    
                }

                // 发送事件
                if state_guard.enable_event_publish {
                    let event = Event::Strategy(StrategyEvent::NodeMessage(message));
                    if let Err(_) = state_guard.event_publisher.publish(event.into()) {
                        tracing::error!(
                            node_id = %state_guard.node_id,
                            "数据源节点发送数据事件失败"
                        );
                    }
                }

            }
            _ => {}
        }
    }

    async fn handle_response_event(state: Arc<RwLock<LiveDataNodeState>>, response_event: ResponseEvent) {
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
            ResponseEvent::MarketDataEngine(MarketDataEngineResponse::SubscribeKlineStreamSuccess(subscribe_kline_stream_success_response)) => {
                
                if request_id == subscribe_kline_stream_success_response.response_id {
                    {
                        let mut state_guard = state.write().await;
                        tracing::info!("{}: K线流订阅成功: {:?}, 开始推送数据", state_guard.node_id, subscribe_kline_stream_success_response);
                        state_guard.request_id = None;
                        //******这里确定数据订阅成功，将状态设置为running
                        tracing::info!("{}: 启动完成", state_guard.node_id);

                    }
                   
                    // 修改状态为running
                    tokio::task::spawn_blocking(move || {
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            // 启动节点
                            Self::update_run_state(state.clone(), NodeStateTransitionEvent::StartComplete).await.unwrap();
            
                        })
                    });
                }
            }
            ResponseEvent::MarketDataEngine(MarketDataEngineResponse::UnsubscribeKlineStreamSuccess(unsubscribe_kline_stream_success_response)) => {
                if request_id == unsubscribe_kline_stream_success_response.response_id {
                    {
                        let mut state_guard = state.write().await;
                        tracing::info!("{}: K线流取消订阅成功: {:?}, 停止推送数据", state_guard.node_id, unsubscribe_kline_stream_success_response);
                        state_guard.request_id = None;
                        //******这里确定数据已经取消订阅，将状态设置为stopped
                        // 节点生命周期结束

                    }

                    Self::cancel_task(state.clone()).await; 
                    
                    tokio::task::spawn_blocking(move || {
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            
                            Self::update_run_state(state.clone(), NodeStateTransitionEvent::StopComplete).await.unwrap();
                        })
                    });
                    
                }
            }   
            _ => {}
        }
    }
    
    pub async fn init_node(self) -> Self {
        self.init_node_sender().await
    }

    async fn init_node_sender(self) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let live_data_node_sender = NodeSender::new(self.state.read().await.node_id.clone(), "live_data_node_output".to_string(), tx);
        self.state.write().await.output_handle.insert("live_data_node_output".to_string(), NodeOutputHandle {
            handle_id: "live_data_node_output".to_string(),
            sender: live_data_node_sender,
            connect_count: 0,
        });
        self
    }

    async fn cancel_task(state: Arc<RwLock<LiveDataNodeState>>) {
        let state_guard = state.read().await;
        state_guard.cancel_token.cancel();
        tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", state_guard.node_id, state_guard.run_state_manager.current_state());
    }

}

#[async_trait]
impl NodeTrait for LiveDataNode {
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

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.output_handle.get(&handle_id).unwrap().sender.clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.output_handle.get("live_data_node_output").unwrap().sender.clone()
    }

    async fn add_message_receiver(&mut self, receiver: NodeReceiver) {
        self.message_receivers.push(receiver);
    }

    async fn add_from_node_id(&mut self, from_node_id: String) {
        self.from_node_id.push(from_node_id);
    }

    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender) {
        self.state.write().await.output_handle.insert(handle_id.clone(), NodeOutputHandle {
            handle_id: handle_id.clone(),
            sender: sender.clone(),
            connect_count: 0,
        });
    }

    async fn add_node_output_handle_connect_count(&mut self, handle_id: String) {
        self.state.write().await.output_handle.get_mut(&handle_id).unwrap().connect_count += 1;
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


    // 之后改为setup
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
        Ok(())
    }

    async fn stop(&mut self) -> Result<(), String> {
        let state = self.state.clone();
        tracing::info!("{}: 开始停止", state.read().await.node_id);
        Self::update_run_state(state.clone(), NodeStateTransitionEvent::Stop).await.unwrap();
        Ok(())
    }

    
}