use types::market::{Exchange, KlineInterval};
use tokio::sync::broadcast;
use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;
use std::error::Error;
use async_trait::async_trait;
use utils::get_utc8_timestamp;
use event_center::Event;
use event_center::market_event::MarketEvent;
use crate::*;
use crate::message::{KlineSeriesMessage, NodeMessage};



#[derive(Debug)]
pub struct DataSourceNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub sender: NodeSender,
    pub receivers: Vec<NodeReceiver>,
    pub market_event_receiver: broadcast::Receiver<Event>,
}

impl Clone for DataSourceNode {
    fn clone(&self) -> Self {
        DataSourceNode { 
            id: self.id, 
            name: self.name.clone(), 
            node_type: self.node_type.clone(), 
            exchange: self.exchange.clone(), 
            symbol: self.symbol.clone(), 
            interval: self.interval.clone(), 
            sender: self.sender.clone(), 
            receivers: self.receivers.clone(), 
            market_event_receiver: self.market_event_receiver.resubscribe() }
    }
}

impl DataSourceNode {
    pub fn new(name: String, exchange: Exchange, symbol: String, interval: KlineInterval, market_event_receiver: broadcast::Receiver<Event>) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_id = Uuid::new_v4();
        Self { id: node_id, name, node_type: NodeType::DataSource, exchange, symbol, interval, sender: NodeSender::new(node_id.to_string(), tx), receivers: Vec::new(), market_event_receiver }
    }
}

#[async_trait]
impl NodeTrait for DataSourceNode {
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
        self.sender.clone()
    }
    fn get_ref_sender(&mut self) -> &mut NodeSender {
        &mut self.sender
    }
    fn push_receiver(&mut self, receiver: NodeReceiver) {
        self.receivers.push(receiver);
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("DataSourceNode run");
        // 监听市场事件
        while let Ok(event) = self.market_event_receiver.recv().await {
            println!("market_event: {:?}", event);

            match event {
                Event::Market(market_event) => {
                    match market_event {
                        MarketEvent::KlineSeriesUpdate(kline_series_update) => {
                            let kline_series_message = KlineSeriesMessage {
                                from_node_id: self.id,
                                from_node_name: self.name.clone(),
                                exchange: kline_series_update.exchange,
                                symbol: kline_series_update.symbol,
                                interval: kline_series_update.interval,
                                kline_series: kline_series_update.kline_series.clone(),
                                batch_id: kline_series_update.batch_id.clone(),
                                message_timestamp: get_utc8_timestamp(),
                            };
                            let message = NodeMessage::KlineSeries(kline_series_message);
                            match self.sender.send(message.clone()) {
                                Ok(receiver_count) => {
                                    println!("+++++++++++++++++++++++++++++++");
                                    println!("批次id: {}", kline_series_update.batch_id);
                                    println!(
                                        "数据源节点发送数据: {:?} 发送成功", 
                                        message, 
                                    );
                                },
                                Err(e) => {
                                    println!(
                                        "数据源节点发送数据: {:?} 发送失败: 错误 = {:?}, 接收者数量 = {}", 
                                        message,
                                        e,
                                        self.sender.receiver_count()
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}