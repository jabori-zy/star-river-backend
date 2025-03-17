use event_center::command_event::{SubscribeKlineStreamParams, MarketDataEngineCommand, CommandEvent};
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
use crate::message::{KlineSeriesMessage, NodeMessage};
use tokio::sync::RwLock;
use std::sync::Arc;
use uuid::Uuid;
use event_center::EventPublisher;
use tokio::sync::mpsc;
use event_center::response_event::{MarketDataEngineResponse, ResponseEvent};
use std::collections::HashMap;

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
    pub data_subscribed: bool,
    pub is_running: bool,
    pub node_output_handle: HashMap<String, NodeSender>, // 节点的出口 {handle_id: sender}, 每个handle对应一个sender
}


#[derive(Debug)]
pub struct LiveDataNode {
    pub state: Arc<RwLock<LiveDataNodeState>>,
    pub node_receivers: Vec<NodeReceiver>, // 接收来自其他节点的数据
    pub from_node_id: Vec<String>, // 来自哪个节点的id
    pub node_type: NodeType,
    pub market_event_receiver: broadcast::Receiver<Event>, // 接收来自市场的数据
    pub response_event_receiver: broadcast::Receiver<Event>, // 接收来自其他节点的数据
    pub event_publisher: EventPublisher, // 发送数据到其他节点
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
            event_publisher: self.event_publisher.clone(),
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
        response_event_receiver: broadcast::Receiver<Event>
    ) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        Self { 
            node_type: NodeType::DataSourceNode, 
            node_receivers: Vec::new(),
            from_node_id: Vec::new(),
            market_event_receiver,
            response_event_receiver,
            event_publisher,
            state: Arc::new(RwLock::new(LiveDataNodeState { 
                strategy_id,
                node_id: node_id.clone(), 
                node_name: name, 
                exchange, 
                symbol, 
                interval, 
                request_id: None,
                data_subscribed: false,
                is_running: false,
                node_output_handle: HashMap::new(),
            })), 
        }
    }

    async fn subscribe_kline_stream(&self) -> Result<(), Box<dyn Error>> {
        let mut state = self.state.write().await;
        let request_id = Uuid::new_v4();
        let params = SubscribeKlineStreamParams {
            strategy_id: state.strategy_id.clone(),
            node_id: state.node_id.clone(),
            exchange: state.exchange.clone(),
            symbol: state.symbol.clone(),
            interval: state.interval.clone(),
            sender: state.node_id.clone(),
            timestamp: get_utc8_timestamp_millis(),
            request_id: request_id,
        };

        state.request_id = Some(request_id);

        let command_event = CommandEvent::MarketDataEngine(MarketDataEngineCommand::SubscribeKlineStream(params));
        tracing::info!("{}订阅k线流: {:?}", state.node_id, command_event);
        self.event_publisher.publish(command_event.into()).unwrap();
        Ok(())
    }

    async fn listen(&self, internal_tx: mpsc::Sender<Event>) -> Result<(), Box<dyn Error>> {
        let mut response_event_receiver = self.response_event_receiver.resubscribe();
        let mut market_event_receiver = self.market_event_receiver.resubscribe();
        tokio::spawn(async move {
            loop {
                tokio::select! {
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
    // 处理接收到的事件
    async fn handle_events(state: Arc<RwLock<LiveDataNodeState>>, mut internal_rx: mpsc::Receiver<Event>) {
        tokio::spawn(async move {
            loop {
                let event = internal_rx.recv().await.unwrap();
                match event {
                    Event::Response(response_event) => {
                        // 处理接收到的事件
                        Self::handle_response_event(state.clone(), response_event).await;
                    }
                    Event::Market(market_event) => {
                        // 处理接收到的行情数据
                        Self::handle_market_event(state.clone(), market_event).await;
                    }
                    _ => {}
                }
            }
        });
    }
    async fn handle_market_event(state: Arc<RwLock<LiveDataNodeState>>, market_event: MarketEvent) {
        // 先获取读锁，检查状态
        let state_guard = state.read().await;
        
        // 判断是否正在运行
        if !state_guard.is_running {
            return;
        }

        // 判断数据是否订阅成功
        if !state_guard.data_subscribed {
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
                let default_node_sender = state_guard.node_output_handle.get("live_data_node_output").expect("实时数据节点默认的消息发送器不存在");
                match default_node_sender.send(message.clone()) {
                    Ok(receiver_count) => {
                        // tracing::info!("+++++++++++++++++++++++++++++++");
                        // tracing::info!("批次id: {}", kline_series_update.batch_id);
                        // tracing::info!("+++++++++++++++++++++++++++++++");
                        // tracing::info!(
                        //     "数据源节点{}发送数据: {:?} 发送成功, 接收者数量 = {}", 
                        //     state_guard.node_id,
                        //     message, 
                        //     receiver_count
                        // );
                    },
                    Err(e) => {
                        tracing::error!(
                            "数据源节点{}发送数据: {:?} 发送失败: 错误 = {:?}, 接收者数量 = {}", 
                            state_guard.node_id,
                            message,
                            e,
                            default_node_sender.receiver_count()
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
                    state_guard.data_subscribed = true;
                    
                    state_guard.request_id = None;
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
        self.state.write().await.node_output_handle.insert("live_data_node_output".to_string(), live_data_node_sender);
        self
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


    async fn get_node_name(&self) -> String {
        self.state.read().await.node_name.clone()
    }

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.node_output_handle.get("live_data_node_output").unwrap().clone()
    }

    fn add_message_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }

    fn add_from_node_id(&mut self, from_node_id: String) {
        self.from_node_id.push(from_node_id);
    }

    async fn add_node_output_handle(&mut self, handle_id: String, sender: NodeSender) {
        self.state.write().await.node_output_handle.insert(handle_id, sender);
    }



    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        tracing::info!("LiveDataNode run");
        {
            let mut state_guard = self.state.write().await;
            state_guard.is_running = true;
        }

        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);

        self.listen(internal_tx).await?;

        let state = self.state.clone();
        tokio::spawn(async move {
            Self::handle_events(state, internal_rx).await;
        });
        // 先订阅k线流
        self.subscribe_kline_stream().await?;

        Ok(())
    }
}