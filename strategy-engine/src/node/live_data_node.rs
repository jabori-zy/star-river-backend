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


// 将需要共享的状态提取出来
#[derive(Debug, Clone)]
pub struct LiveDataNodeState {
    pub node_id: String,
    pub node_name: String,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub node_sender: NodeSender,
    
}


#[derive(Debug)]
pub struct LiveDataNode {
    pub state: Arc<RwLock<LiveDataNodeState>>,
    pub node_receivers: Vec<NodeReceiver>,
    pub node_type: NodeType,
    pub market_event_receiver: broadcast::Receiver<Event>,
}

impl Clone for LiveDataNode {
    fn clone(&self) -> Self {
        LiveDataNode { 
            node_type: self.node_type.clone(), 
            node_receivers: self.node_receivers.clone(), 
            market_event_receiver: self.market_event_receiver.resubscribe(), 
            state: self.state.clone(), 
        }
    }
}

impl LiveDataNode {
    pub fn new(node_id: String, name: String, exchange: Exchange, symbol: String, interval: KlineInterval, market_event_receiver: broadcast::Receiver<Event>) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        Self { 
            node_type: NodeType::DataSourceNode, 
            node_receivers: Vec::new(),
            market_event_receiver,
            state: Arc::new(RwLock::new(LiveDataNodeState { 
                node_id: node_id.clone(), 
                node_name: name, 
                exchange, 
                symbol, 
                interval, 
                node_sender: NodeSender::new(node_id, tx), 
            })), 
        }
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

    async fn get_sender(&self) -> NodeSender {
        self.state.read().await.node_sender.clone()
    }

    fn push_receiver(&mut self, receiver: NodeReceiver) {
        self.node_receivers.push(receiver);
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("LiveDataNode run");
        // // 监听市场事件
        // while let Ok(event) = self.market_event_receiver.recv().await {
        //     println!("market_event: {:?}", event);

        //     match event {
        //         Event::Market(market_event) => {
        //             match market_event {
        //                 MarketEvent::KlineSeriesUpdate(kline_series_update) => {
        //                     let kline_series_message = KlineSeriesMessage {
        //                         from_node_id: self.state.read().await.node_id.clone(),
        //                         from_node_name: self.state.read().await.node_name.clone(),
        //                         exchange: kline_series_update.exchange,
        //                         symbol: kline_series_update.symbol,
        //                         interval: kline_series_update.interval,
        //                         kline_series: kline_series_update.kline_series.clone(),
        //                         batch_id: kline_series_update.batch_id.clone(),
        //                         message_timestamp: get_utc8_timestamp_millis(),
        //                     };
        //                     let message = NodeMessage::KlineSeries(kline_series_message);
        //                     let state = self.state.read().await;
        //                     match state.node_sender.send(message.clone()) {
        //                         Ok(receiver_count) => {
        //                             println!("+++++++++++++++++++++++++++++++");
        //                             println!("批次id: {}", kline_series_update.batch_id);
        //                             println!(
        //                                 "数据源节点发送数据: {:?} 发送成功", 
        //                                 message, 
        //                             );
        //                         },
        //                         Err(e) => {
        //                             println!(
        //                                 "数据源节点发送数据: {:?} 发送失败: 错误 = {:?}, 接收者数量 = {}", 
        //                                 message,
        //                                 e,
        //                                 state.node_sender.receiver_count()
        //                             );
        //                         }
        //                     }
        //                 }
        //                 _ => {}
        //             }
        //         }
        //         _ => {}
        //     }
        // }
        Ok(())
    }
}