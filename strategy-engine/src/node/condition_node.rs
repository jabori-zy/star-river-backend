use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use uuid::Uuid;
use std::error::Error;
use async_trait::async_trait;
use tokio::sync::broadcast;
use crate::*;
use serde::{Serialize, Deserialize};
use strum::EnumString;
use std::str::FromStr;
use futures::stream::select_all;
use tokio_stream::wrappers::BroadcastStream;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::RwLock;


#[derive(Debug)]
pub struct ConditionNodeState {
    pub current_batch_id: Option<String>,
    pub is_processing: bool,
}


#[derive(Debug, Clone)]
pub struct ConditionNode {
    pub node_id: i32,
    pub name: String,
    pub condition_type: ConditionType,
    pub conditions: Vec<Condition>,
    pub input_values: HashMap<Uuid, NodeMessage>,
    pub sender: NodeSender,
    pub receivers: Vec<NodeReceiver>,
    pub state: Arc<RwLock<ConditionNodeState>>,
}

#[async_trait]
impl NodeTrait for ConditionNode {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn NodeTrait> {
        Box::new(self.clone())
    }

    async fn get_sender(&self) -> NodeSender {
        self.sender.clone()
    }

    fn push_receiver(&mut self, receiver: NodeReceiver) {  
        self.receivers.push(receiver);
    }


    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("条件节点开始运行");
        self.listen_message().await;

        //     // let is_new_batch = self.update_value(message.from_node_id, message.value, message.batch_id.clone());
        //     self.update_value(message.from_node_id, message.message, message.batch_id.clone());

        //     // 检查是否收集完整
        //     if self.is_batch_complete()  && !self.is_processing {
        //         println!("开始处理批次 {}", self.current_batch_id.as_ref().unwrap());
        //         self.is_processing = true;
        //         // let result = self.evaluate();
        //         // println!("批次{}信号结果: {}", self.current_batch_id.as_ref().unwrap(), result);

        //         // 完成处理
        //         println!("批次处理完成");
        //         println!("+++++++++++++++++++++++++++++++");
        //         self.is_processing = false;
                
        //     }
        //     else if self.is_processing {
        //         println!("新数据到达时正在处理中，跳过当前处理");
        //     }

        //     // tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        // }
        Ok(())
        
    }
}



impl ConditionNode {

    pub fn new(node_id: i32, name: String, condition_type: ConditionType) -> Self {
        let (tx, _) = broadcast::channel::<NodeMessage>(100);
        Self { 
            node_id, 
            name, 
            condition_type, 
            conditions: vec![],
            input_values: HashMap::new(), 
            sender: NodeSender::new(node_id.to_string(), tx), 
            receivers: Vec::new(),
            state: Arc::new(RwLock::new(ConditionNodeState {
                current_batch_id: None,
                is_processing: false,
            })),
        }
    }

    pub fn add_condition(&mut self, condition: Condition) {
        match &mut self.condition_type {
            ConditionType::And => self.conditions.push(condition),
            ConditionType::Or => self.conditions.push(condition),
        }
    }

    pub fn get_condition(&self) -> Vec<Condition> {
        self.conditions.clone()
    }

    pub async fn listen_message(&mut self) {
        let streams: Vec<_> = self.receivers.iter()
            .map(|receiver| BroadcastStream::new(receiver.get_receiver()))
            .collect();
        let mut combined_stream = select_all(streams);
        while let Some(result) = combined_stream.next().await {
            let message = result.unwrap();
            match message {
                NodeMessage::Indicator(indicator_message) => {
                    println!("条件节点收到数据: batch_id={}, from={}, message={:?}", 
                        indicator_message.batch_id, indicator_message.from_node_name, indicator_message.data);
                }
                _ => {}
            }
        }

    }

    // pub fn get_input_values(&self) -> HashMap<Uuid, Message> {
    //     self.input_values.clone()
    // }

    // fn update_value(&mut self, from_node_id: Uuid, message: Message, batch_id: String) {
    //     // 收到新批次数据时，直接清空旧数据开始新的处理
    //     if self.current_batch_id.as_ref() != Some(&batch_id) {
    //         println!("收到新批次数据，清空旧数据");
    //         self.input_values.clear();
    //         self.current_batch_id = Some(batch_id);
    //         self.is_processing = false;  // 重置处理状态
    //     }
        
    //     self.input_values.insert(from_node_id, message);
    // }

    // 检查当前批次是否接收完整
    fn is_batch_complete(&self) -> bool {
        self.input_values.len() == self.receivers.len()
    }



    // 评估单个条件
    // fn evaluate_single_condition(&self, condition: &Condition) -> bool {
    //     let left_value = match self.input_values.get(&condition.left_value_node_id) {
    //         Some(value) => *value,
    //         None => return false, // 如果没有值，条件判断失败
    //     };

    //     let right_value = match self.input_values.get(&condition.right_value_node_id) {
    //         Some(value) => *value,
    //         None => return false,
    //     };
    //     true

        // match condition.operator {
        //     Operator::GreaterThan => left_value > right_value,
        //     Operator::LessThan => left_value < right_value,
        //     Operator::Equal => (left_value - right_value).abs() < f64::EPSILON,
        //     Operator::GreaterThanOrEqual => left_value >= right_value,
        //     Operator::LessThanOrEqual => left_value <= right_value,
        //     Operator::NotEqual => left_value != right_value,
        // }
    // }

    // pub fn evaluate(&self) -> bool {
    //     match &self.condition_type {
    //         ConditionType::And => {
    //             self.conditions.iter().all(|condition| self.evaluate_single_condition(condition))
    //         }
    //         ConditionType::Or => {
    //             self.conditions.iter().any(|condition| self.evaluate_single_condition(condition))
    //         }
    //     }
    // }


}



#[derive(Debug, Clone, Serialize, Deserialize, EnumString)]
pub enum Operator {
    #[serde(rename = ">")]
    #[strum(serialize = ">")]
    GreaterThan,
    #[serde(rename = "<")]
    #[strum(serialize = "<")]
    LessThan,
    #[serde(rename = "=")]
    #[strum(serialize = "=")]
    Equal,
    #[serde(rename = "!=")]
    #[strum(serialize = "!=")]
    NotEqual,
    #[serde(rename = ">=")]
    #[strum(serialize = ">=")]
    GreaterThanOrEqual,
    #[serde(rename = "<=")]
    #[strum(serialize = "<=")]
    LessThanOrEqual,
}

#[derive(Debug, Clone)]
pub struct Condition {
    // 左边值的节点id
    pub left_value_node_id: Uuid,
    // 操作符
    pub operator: Operator,
    // 右边值的节点id
    pub right_value_node_id: Uuid,
}

impl Condition {
    pub fn new(left_value_node_id: Uuid, operator: &str, right_value_node_id: Uuid) -> Self {
        let operator = Operator::from_str(operator).expect("Invalid operator");
        Self { left_value_node_id, operator, right_value_node_id }
    }
}

#[derive(Debug, Clone)]
pub enum ConditionType {
    And,  // 所有条件都必须满足
    Or,   // 任意条件满足即可
}