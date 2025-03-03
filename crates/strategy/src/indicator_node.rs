use types::indicator::Indicators;
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use std::error::Error;
use async_trait::async_trait;
use futures::StreamExt;
use types::market::{Exchange, KlineInterval};
use crate::*;
use event_center::{Event, EventPublisher};
use event_center::command_event::{CalculateIndicatorParams, CommandEvent, IndicatorEngineCommand};
use event_center::response_event::{ResponseEvent, IndicatorEngineResponse};
use utils::get_utc8_timestamp;
use crate::message::IndicatorMessage;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;

// 将需要共享的状态提取出来
#[derive(Debug, Clone)]
pub struct IndicatorNodeState { 
    pub current_batch_id: Option<String>,
    pub request_id: Option<Uuid>,


}

// 指标节点
#[derive(Debug)]
pub struct IndicatorNode {
    pub id: Uuid,
    pub name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub node_type: NodeType,
    pub indicator: Indicators,
    pub node_sender: NodeSender,
    pub node_receivers: Vec<NodeReceiver>,
    pub event_publisher: EventPublisher,
    pub response_event_receiver: broadcast::Receiver<Event>,
    // pub current_batch_id: Option<String>,
    // pub request_id: Option<Uuid>,
    pub state: Arc<RwLock<IndicatorNodeState>>,
}

impl Clone for IndicatorNode {
    fn clone(&self) -> Self {
        Self { 
            id: self.id.clone(), 
            name: self.name.clone(), 
            exchange: self.exchange.clone(), 
            symbol: self.symbol.clone(), 
            interval: self.interval.clone(), 
            node_type: self.node_type.clone(), 
            indicator: self.indicator.clone(),
            node_sender: self.node_sender.clone(),
            node_receivers: self.node_receivers.clone(),
            event_publisher: self.event_publisher.clone(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
            // current_batch_id: self.current_batch_id.clone(),
            // request_id: self.request_id.clone(),
            state: self.state.clone(),
        }
    }
}




impl IndicatorNode {
    pub fn new(name: String, exchange: Exchange, symbol: String, interval: KlineInterval, indicator: Indicators, event_publisher: EventPublisher, response_event_receiver: broadcast::Receiver<Event>) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_id = Uuid::new_v4();
        Self { 
            id: node_id, 
            name, 
            exchange,
            symbol,
            interval,
            node_type: NodeType::Indicator, 
            indicator, 
            node_sender: NodeSender::new(node_id.to_string(), tx), 
            node_receivers: Vec::new(),
            event_publisher,
            response_event_receiver,
            // current_batch_id: None,
            // request_id: None,
            state: Arc::new(RwLock::new(IndicatorNodeState {
                current_batch_id: None,
                request_id: None,
            })),
        }
    }

    // 监听事件
    async fn listen_events(&mut self,
        internal_tx: mpsc::Sender<Event>,
    ) -> Result<(), Box<dyn Error>> {
        // 接收指标引擎返回的计算结果，并发送给下一个节点
        // 指标引擎响应的接收器
        let mut response_event_receiver = self.response_event_receiver.resubscribe();
        tokio::spawn(async move {
            while let Ok(response_event) = response_event_receiver.recv().await {   
                let _ = internal_tx.send(response_event).await;
            }
        });
        
        Ok(())
    }

    // 处理接收到的事件
    async fn handle_events(state: Arc<RwLock<IndicatorNodeState>>, mut internal_rx: mpsc::Receiver<Event>) {
        loop {
            let event = internal_rx.recv().await.unwrap();
            match event {
                Event::Response(response_event) => {
                    IndicatorNode::handle_response_event(response_event).await;
                }
                _ => {}
            }
        }
    }

    async fn handle_response_event(response_event: ResponseEvent) {
        match response_event {
            ResponseEvent::IndicatorEngine(indicator_engine_response) => {
                tracing::info!("指标节点接收到指标引擎返回的计算结果: {:?}", indicator_engine_response);
            }
            _ => {}
        }
    }
    
    // 处理指标引擎返回的计算结果
    async fn handle_indicator_engine_response(state: Arc<RwLock<IndicatorNodeState>>, indicator_engine_response: IndicatorEngineResponse) {
        tracing::info!("指标节点接收到指标引擎返回的计算结果: {:?}", indicator_engine_response);
        match indicator_engine_response {
            IndicatorEngineResponse::CalculateIndicatorFinish(calculate_indicator_response) => {
                tracing::info!("指标节点接收到指标引擎返回的计算结果: {:?}", calculate_indicator_response);

            }
            _ => {}
        }
    }

    // 监听节点传递过来的message
    async fn listen_message(&mut self) {
        let indicator = self.indicator.clone();
        let node_name = self.name.clone();
        let node_id = self.id.clone();
        let event_publisher = self.event_publisher.clone();

        // 更新 node state 的 request_id
        {
            let mut state_guard = self.state.write().await;
            state_guard.request_id = Some(Uuid::new_v4());
        }  // 锁在这里自动释放

        // 创建一个流，用于接收节点传递过来的message
        let streams: Vec<_> = self.node_receivers.iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();
        let mut combined_stream = select_all(streams);

        let state = self.state.clone();
        // 指标节点接收数据源节点的数据
        tokio::spawn(async move {
            while let Some(receive_message) = combined_stream.next().await {
                if let Ok(receive_message) = receive_message {
                    println!("指标节点{}接收到数据: {:?}", node_name, receive_message);
                    match receive_message {
                        NodeMessage::KlineSeries(kline_series_message) => {
                            // 向指标引擎发送计算指令
                            let request_id = {
                                let state_guard = state.write().await;
                                state_guard.request_id.clone().unwrap()
                            };// 锁在这里自动释放
                            
                            let calculate_indicator_params = CalculateIndicatorParams {
                                exchange: kline_series_message.exchange,
                                symbol: kline_series_message.symbol,
                                interval: kline_series_message.interval,
                                indicator: indicator.clone(),
                                kline_series: kline_series_message.kline_series,
                                sender: node_id.to_string(),
                                command_timestamp: get_utc8_timestamp(),
                                request_id: request_id,
                                batch_id: kline_series_message.batch_id,
                            };
                            let event = Event::Command(CommandEvent::IndicatorEngine(IndicatorEngineCommand::CalculateIndicator(calculate_indicator_params)));
                            event_publisher.publish(event).unwrap();
                        }
                        _ => {}
                    }
                }
            }
        });
    }
}

#[async_trait]
impl NodeTrait for IndicatorNode {
    fn id(&self) -> Uuid {
        self.id
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }
    fn get_sender(&self) -> NodeSender {
        self.node_sender.clone()
    }
    fn get_ref_sender(&mut self) -> &mut NodeSender {
        &mut self.node_sender
    }
    fn push_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }
    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("IndicatorNode run");
        // 创建内部通信通道
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);

        // 启动监听
        self.listen_events(internal_tx).await?;

        let state = self.state.clone();
        tokio::spawn(async move {
            IndicatorNode::handle_events(state, internal_rx).await;
        });

        // 接收节点传递过来的message
        self.listen_message().await;
        Ok(())
    }
}