use event_center::command_event::{SubscribeKlineStreamParams, MarketDataEngineCommand, CommandEvent, UnsubscribeKlineStreamParams};
use types::market::{Exchange, KlineInterval};
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use std::error::Error;
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
use event_center::response_event::{MarketDataEngineResponse, ResponseEvent};
use event_center::strategy_event::StrategyEvent;
use std::collections::HashMap;
use crate::NodeOutputHandle;
use tokio::sync::watch;
use crate::strategy::StrategyState;
use crate::node::NodeRunState;
use crate::node::NodeTrait;
use tokio_util::sync::CancellationToken;

// 将需要共享的状态提取出来
#[derive(Debug, Clone)]
pub struct LiveDataNodeState {
    pub strategy_id: i32,
    pub node_id: String,
    pub node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub request_id: Option<Uuid>,
    pub node_run_state: NodeRunState,
    pub output_handle: HashMap<String, NodeOutputHandle>, // 节点的出口连接数 {handle_id: count}, 每个handle对应一个连接数
    pub event_publisher: EventPublisher, // 发送数据事件中心
    pub enable_event_publish: bool, // 是否启用事件发布
    pub cancel_token: CancellationToken,
}

#[derive(Debug)]
pub struct LiveDataNode {
    pub state: Arc<RwLock<LiveDataNodeState>>,
    pub node_receivers: Vec<NodeReceiver>, // 接收来自其他节点的数据
    pub from_node_id: Vec<String>, // 来自哪个节点的id
    pub node_type: NodeType,
    pub market_event_receiver: broadcast::Receiver<Event>, // 接收来自市场通道的数据
    pub response_event_receiver: broadcast::Receiver<Event>, // 接收来自响应通道的数据
    strategy_state_rx: watch::Receiver<StrategyState>,
}

impl Clone for LiveDataNode {
    fn clone(&self) -> Self {
        LiveDataNode { 
            node_type: self.node_type.clone(),
            from_node_id: self.from_node_id.clone(),
            node_receivers: self.node_receivers.clone(), 
            response_event_receiver: self.response_event_receiver.resubscribe(),
            market_event_receiver: self.market_event_receiver.resubscribe(), 
            state: self.state.clone(),
            strategy_state_rx: self.strategy_state_rx.clone(),
        }
    }
}

impl LiveDataNode {
    pub fn new(
        strategy_id: i32, 
        node_id: String, 
        name: String, 
        exchange: Exchange, 
        symbol: String, 
        interval: KlineInterval, 
        event_publisher: EventPublisher, 
        market_event_receiver: broadcast::Receiver<Event>,
        response_event_receiver: broadcast::Receiver<Event>,
        strategy_state_rx: watch::Receiver<StrategyState>,
    ) -> Self {
        let cancel_token = CancellationToken::new();
        Self { 
            node_type: NodeType::DataSourceNode, 
            node_receivers: Vec::new(),
            from_node_id: Vec::new(),
            market_event_receiver,
            response_event_receiver,
            strategy_state_rx,
            state: Arc::new(RwLock::new(LiveDataNodeState {
                strategy_id,
                node_id: node_id.clone(), 
                node_name: name, 
                exchange, 
                symbol, 
                interval, 
                request_id: None,
                node_run_state: NodeRunState::Created,
                output_handle: HashMap::new(),
                event_publisher,
                enable_event_publish: false,
                cancel_token,
            }
        )), 
        }
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
            sender: state_guard.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        state_guard.request_id = Some(request_id);

        let command_event = CommandEvent::MarketDataEngine(MarketDataEngineCommand::SubscribeKlineStream(params));
        tracing::info!("{}订阅k线流: {:?}", state_guard.node_id, command_event);
        if let Err(_) = state_guard.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %state_guard.node_id,
                "数据源节点发送数据失败"
            );
        }
        Ok(())
    }

    async fn unsubscribe_kline_stream(state: Arc<RwLock<LiveDataNodeState>>) -> Result<(), Box<dyn Error>> {
        let mut state_guard = state.write().await;
        let request_id = Uuid::new_v4();
        let params = UnsubscribeKlineStreamParams {
            strategy_id: state_guard.strategy_id.clone(),
            node_id: state_guard.node_id.clone(),
            exchange: state_guard.exchange.clone(),
            symbol: state_guard.symbol.clone(),
            interval: state_guard.interval.clone(),
            sender: state_guard.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        // 设置请求id
        state_guard.request_id = Some(request_id);

        let command_event = CommandEvent::MarketDataEngine(MarketDataEngineCommand::UnsubscribeKlineStream(params));
        if let Err(e) = state_guard.event_publisher.publish(command_event.into()) {
            tracing::error!(
                node_id = %state_guard.node_id,
                "数据源节点发送数据失败"
            );
        }   
        Ok(())
    }


    // 监听策略状态
    async fn listen_strategy_signal(&self, state: Arc<RwLock<LiveDataNodeState>>) -> Result<(), String> {
        let mut strategy_state_rx = self.strategy_state_rx.clone();
        let cancel_token = state.read().await.cancel_token.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = cancel_token.cancelled() => {
                        tracing::info!("{} 节点监听策略状态进程已中止", state.read().await.node_id);
                        break;
                    }

                    signal = strategy_state_rx.changed() => {

                        if signal.is_err() {
                            tracing::error!("{} 监听策略状态失败", state.read().await.node_id);
                            break;
                        }

                        let strategy_state = strategy_state_rx.borrow().clone();
                        match strategy_state {
                            StrategyState::Running => {
                                tracing::debug!("节点: {} 收到启动信号", state.read().await.node_id);
                                Self::start(state.clone()).await.unwrap();
                            }
                            StrategyState::Stopping => {
                                // 收到停止信号后，将状态修改为stopping
                                {
                                    let mut state_guard = state.write().await;
                                    state_guard.node_run_state = NodeRunState::Stopping;
                                    tracing::debug!("节点: {} 收到停止信号, 开始停止，当前节点状态: {:?}", state_guard.node_id, state_guard.node_run_state);
                                }
                                
                                Self::stop(state.clone()).await.unwrap();
                            }
                            _ => {}

                        }
                    }
                }
            }
            
        });
        Ok(())
    }

    // 监听外部事件
    async fn listen_external_events(&self, internal_tx: mpsc::Sender<Event>) -> Result<(), Box<dyn Error>> {
        let mut response_event_receiver = self.response_event_receiver.resubscribe();
        let mut market_event_receiver = self.market_event_receiver.resubscribe();
        let cancel_token = self.state.read().await.cancel_token.clone();
        let node_id = self.state.read().await.node_id.clone();
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
    async fn handle_external_events(&self, state: Arc<RwLock<LiveDataNodeState>>, mut internal_rx: mpsc::Receiver<Event>) {
        let cancel_token = self.state.read().await.cancel_token.clone();
        let node_id = self.state.read().await.node_id.clone();
        
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

        if state_guard.node_run_state != NodeRunState::Running {
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
                tracing::info!("{}: 发送数据: {:?}", state_guard.node_id, message);
                // 获取handle的连接数
                let default_handle_connect_count = state_guard.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在").connect_count;
                // 如果连接数为0，则不发送数据
                if default_handle_connect_count > 0 {
                    let default_node_sender = state_guard.output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在");
                    match default_node_sender.sender.send(message.clone()) {
                        Ok(_) => (),
                        Err(e) => tracing::error!(
                            node_id = %state_guard.node_id,
                            message = ?message,
                            error = ?e,
                            receiver_count = default_node_sender.connect_count,
                                "数据源节点发送数据失败"
                            ),
                        }
                    
                }

                // 发送事件
                if state_guard.enable_event_publish {
                    let event = Event::Strategy(StrategyEvent::NodeMessage(message));
                    if let Err(e) = state_guard.event_publisher.publish(event.into()) {
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
        match response_event {
            ResponseEvent::MarketDataEngine(MarketDataEngineResponse::SubscribeKlineStreamSuccess(subscribe_kline_stream_success_response)) => {
                let mut state_guard = state.write().await;
                let request_id = match state_guard.request_id {
                    Some(id) => id,
                    None => return,
                };

                if request_id == subscribe_kline_stream_success_response.response_id {
                    tracing::info!("{}: K线流订阅成功: {:?}, 开始推送数据", state_guard.node_id, subscribe_kline_stream_success_response);
                    state_guard.request_id = None;
                    //******这里确定数据订阅成功，将状态设置为running
                    state_guard.node_run_state = NodeRunState::Running;
                }
            }
            ResponseEvent::MarketDataEngine(MarketDataEngineResponse::UnsubscribeKlineStreamSuccess(unsubscribe_kline_stream_success_response)) => {
                let mut state_guard = state.write().await;
                let request_id = match state_guard.request_id {
                    Some(id) => id,
                    None => return,
                };

                if request_id == unsubscribe_kline_stream_success_response.response_id {
                    tracing::info!("{}: K线流取消订阅成功: {:?}, 停止推送数据", state_guard.node_id, unsubscribe_kline_stream_success_response);
                    state_guard.request_id = None;
                    //******这里确定数据已经取消订阅，将状态设置为stopped
                    state_guard.node_run_state = NodeRunState::Stopped;
                    // 节点生命周期结束
                    state_guard.cancel_token.cancel();
                    tracing::info!("{}: 节点已安全停止, 当前节点状态: {:?}", state_guard.node_id, state_guard.node_run_state);
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
    async fn start(state: Arc<RwLock<LiveDataNodeState>>) -> Result<(), String> {
        // 1. 首先检查状态，完成后立即释放锁
        let should_subscribe = {
            let state_guard = state.read().await;
            let node_name = state_guard.node_name.clone();
            let node_id = state_guard.node_id.clone();
            let current_state = state_guard.node_run_state.clone();

            tracing::info!("{}: 开始启动", node_name);
            
            if current_state != NodeRunState::Ready {
                tracing::warn!(
                    node_id = %node_id,
                    current_state = ?current_state,
                    "节点未初始化, 不订阅K线流"
                );
                false
            } else {
                true
            }
        }; // 这里读锁被释放

        // 2. 根据检查结果决定是否订阅
        if should_subscribe {
            Self::subscribe_kline_stream(state.clone()).await?;
        }
        Ok(())
    }

    async fn stop(state: Arc<RwLock<LiveDataNodeState>>) -> Result<(), Box<dyn Error>> {
        
        // 1. 首先检查状态，完成后立即释放锁
        let should_stop = {
            let state_guard = state.read().await;
            let node_name = state_guard.node_name.clone();
            let current_state = state_guard.node_run_state.clone();
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



        Ok(())
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

    fn add_message_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }

    fn add_from_node_id(&mut self, from_node_id: String) {
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

    async fn enable_node_event_publish(&mut self) {
        self.state.write().await.enable_event_publish = true;
    }

    async fn disable_node_event_publish(&mut self) {
        self.state.write().await.enable_event_publish = false;
    }


    // 之后改为setup
    async fn init(&mut self) -> Result<NodeRunState, Box<dyn Error>> {
        tracing::info!("{}: 开始初始化", self.state.read().await.node_id);
        // 判断是否可以初始化
        let should_setup = {
            let state_guard = self.state.read().await;
            if state_guard.node_run_state == NodeRunState::Created {
                true
            } else {
                false
            }
        };

        // 如果不能初始化，则返回错误
        if !should_setup {
            tracing::error!("{}: 节点状态不是Created, 不进行初始化", self.state.read().await.node_id);
            return Err("节点状态不是Created".into());
        }

        // 监听策略状态
        self.listen_strategy_signal(self.state.clone()).await?;

        // 创建内部通道
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);
        // 监听外部事件
        self.listen_external_events(internal_tx).await.map_err(|e| e.to_string())?;

        // 处理内部事件
        let state = self.state.clone();
        self.handle_external_events(state, internal_rx).await;

        // 设置成功，将状态设置为Ready
        let mut state_guard = self.state.write().await;
        state_guard.node_run_state = NodeRunState::Ready;
        tracing::info!("{}: 节点初始化成功, 节点状态: {:?}", state_guard.node_name, state_guard.node_run_state);
        
        Ok(NodeRunState::Ready)
    }

    async fn get_node_run_state(&self) -> NodeRunState {
        self.state.read().await.node_run_state.clone()
    }
}