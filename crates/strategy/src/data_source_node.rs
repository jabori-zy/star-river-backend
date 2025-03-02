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

use crate::node::*;




#[derive(Debug, Clone)]
pub struct DataSourceNode {
    pub id: Uuid,
    pub name: String,
    pub node_type: NodeType,
    pub exchange: Exchange,
    pub symbol: String,
    pub interval: KlineInterval,
    pub sender: NodeSender,
    pub receivers: Vec<NodeReceiver>,
}

impl DataSourceNode {
    pub fn new(name: String, exchange: Exchange, symbol: String, interval: KlineInterval) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        let node_id = Uuid::new_v4();
        Self { id: node_id, name, node_type: NodeType::DataSource, exchange, symbol, interval, sender: NodeSender::new(node_id.to_string(), tx), receivers: Vec::new() }
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
        let mut rng = StdRng::from_entropy();
        loop {
            // 生成随机数据
            let price = rng.gen_range(1.0..100.0);
            let timestamp = get_utc8_timestamp();
            let random = rand::random::<u16>(); 
            let batch_id = format!("{}-{}", timestamp, random);
            let message = NodeMessage {
                from_node_id: self.id,
                from_node_name: self.name.clone(),
                value: price,
                message_type: MessageType::Kline,
                batch_id: batch_id.clone(),
                timestamp: timestamp,
            };
            
            // 发送到所有输出通道
            match self.sender.send(message) {
                Ok(receiver_count) => {
                    println!("+++++++++++++++++++++++++++++++");
                    println!("批次id: {}", batch_id);
                    println!(
                        "数据源节点发送数据: {:?} 发送成功", 
                        price, 
                    );
                },
                Err(e) => {
                    println!(
                        "价格 {:?} 发送失败: 错误 = {:?}, 接收者数量 = {}", 
                        price,
                        e,
                        self.sender.receiver_count()
                    );
                }
            }
    
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
}