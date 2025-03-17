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
use utils::get_utc8_timestamp_millis;
use crate::message::IndicatorMessage;
use tokio::sync::mpsc;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;

// 将需要共享的状态提取出来
#[derive(Debug, Clone)]
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
}

// 指标节点
#[derive(Debug)]
pub struct IndicatorNode {
    pub node_type: NodeType,
    pub node_receivers: Vec<NodeReceiver>,
    pub from_node_id: Vec<String>,

    pub event_publisher: EventPublisher,
    pub response_event_receiver: broadcast::Receiver<Event>,
    pub state: Arc<RwLock<IndicatorNodeState>>,
}

impl Clone for IndicatorNode {
    fn clone(&self) -> Self {
        Self {
            node_type: self.node_type.clone(), 
            node_receivers: self.node_receivers.clone(),
            from_node_id: self.from_node_id.clone(),
            event_publisher: self.event_publisher.clone(),
            response_event_receiver: self.response_event_receiver.resubscribe(),
            state: self.state.clone(),
        }
    }
}




impl IndicatorNode {
    pub fn new(strategy_id: i32, node_id: String, node_name: String, exchange: Exchange, symbol: String, interval: KlineInterval, indicator: Indicators, event_publisher: EventPublisher, response_event_receiver: broadcast::Receiver<Event>) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        Self { 
            node_type: NodeType::IndicatorNode,
            node_receivers: Vec::new(),
            from_node_id: Vec::new(),
            event_publisher,
            response_event_receiver,
            state: Arc::new(RwLock::new(IndicatorNodeState {
                strategy_id,
                node_id: node_id.clone(),
                node_name,
                exchange,
                symbol,
                interval,
                indicator,
                current_batch_id: None,
                request_id: None,
                node_output_handle: HashMap::new(),
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
        tokio::spawn(async move {
            loop {
                let event = internal_rx.recv().await.unwrap();
                match event {
                Event::Response(response_event) => {
                    // 处理接收到的事件
                    IndicatorNode::handle_response_event(state.clone(), response_event).await;
                }
                    _ => {}
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
                    let default_node_sender = state_guard.node_output_handle.get("indicator_node_output").expect("指标节点默认的消息发送器不存在");
                    match default_node_sender.send(NodeMessage::Indicator(indicator_message.clone())) {
                        Ok(receiver_count) => {
                            // tracing::info!("节点{}发送指标数据: {:?} 发送成功, 接收者数量 = {}", state_guard.node_id, indicator_message, receiver_count);
                        }
                        Err(e) => {
                            tracing::error!("节点{}发送指标数据失败: 错误 = {:?}, 接收者数量 = {}", state_guard.node_id, e, default_node_sender.receiver_count());
                        }
                    }
                }
            }
        }
    }

    // 监听节点传递过来的message
    async fn listen_message(&mut self) {
        let event_publisher = self.event_publisher.clone();

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
                    // tracing::info!("节点{}接收到数据: {:?}", state.read().await.node_id, receive_message);
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
                            event_publisher.publish(event).unwrap();
                        }
                        _ => {}
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
        self.state.write().await.node_output_handle.insert("indicator_node_output".to_string(), indicator_node_sender);
        self
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

    async fn get_node_sender(&self, handle_id: String) -> NodeSender {
        self.state.read().await.node_output_handle.get(&handle_id).unwrap().clone()
    }

    async fn get_default_node_sender(&self) -> NodeSender {
        self.state.read().await.node_output_handle.get("indicator_node_output").unwrap().clone()
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
        println!("IndicatorNode run");
        // 创建内部通信通道
        let (internal_tx, internal_rx) = tokio::sync::mpsc::channel::<Event>(100);

        // 启动监听
        self.listen_events(internal_tx).await?;

        let state = self.state.clone();
        IndicatorNode::handle_events(state, internal_rx).await;


        // 接收节点传递过来的message
        self.listen_message().await;
        Ok(())
    }
}