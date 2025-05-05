use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp;
use event_center::strategy_event::StrategyEvent;
use event_center::Event;
use super::super::node_context::{BaseNodeContext,NodeContext};
use super::condition::*;
use types::strategy::message::{IndicatorMessage, SignalMessage, Signal, NodeMessage, SignalType};
use super::if_else_node_type::*;
use types::strategy::TradeMode;




#[derive(Debug, Clone)]
pub struct IfElseNodeContext {
    pub base_context: BaseNodeContext,
    pub current_batch_id: Option<String>,
    pub is_processing: bool,
    pub received_flag: HashMap<String, bool>, // 用于记录每个节点的数据是否接收完成
    pub received_value: HashMap<String, Option<NodeMessage>>, // 用于记录每个节点的数据
    // pub cases: Vec<Case>,
    pub live_config: Option<IfElseNodeLiveConfig>,
    pub backtest_config: Option<IfElseNodeBacktestConfig>,
    pub simulate_config: Option<IfElseNodeSimulateConfig>,
    

}



#[async_trait]
impl NodeContext for IfElseNodeContext {
    
    fn clone_box(&self) -> Box<dyn NodeContext> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BaseNodeContext {
        &mut self.base_context
    }


    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let _event = event;
        Ok(())
    }

    async fn handle_message(&mut self, message: NodeMessage) -> Result<(), String> {
        match message {
            NodeMessage::Indicator(indicator_message) => {
                self.handle_indicator_message(indicator_message).await;
            }
            _ => {}
        }
        Ok(())
    }
}

impl IfElseNodeContext {

    // 处理指标消息
    async fn handle_indicator_message(&mut self, indicator_message: IndicatorMessage) {
        // tracing::debug!("{}: 收到指标消息: {:?}", self.base_context.node_id, indicator_message);
        let from_node_id = indicator_message.from_node_id.clone();

        // 接收到信息之后，更新值的槽位
        // 更新接收值
        let received_value = self.received_value.get_mut(&from_node_id).unwrap();
        *received_value = Some(NodeMessage::Indicator(indicator_message));

        // 更新接收标记
        let received_flag = self.received_flag.get_mut(&from_node_id).unwrap();
        *received_flag = true;
    }

    // 初始化接收标记
    pub async fn init_received_flag(&mut self) {
        for from_node_id in self.get_from_node_id().clone() {
            self.received_flag.insert(from_node_id.clone(), false);
        }
        
    }

    pub async fn init_received_value(&mut self) {
        for from_node_id in self.get_from_node_id().clone() {
            self.received_value.insert(from_node_id.clone(), None);
        }
    }



    // 开始评估各个分支
    pub async fn evaluate(&mut self) {
        // 在锁外执行评估
        let mut case_matched = false; // 是否匹配到case
        // 根据交易模式获取case
        let cases = match self.base_context.trade_mode {
            TradeMode::Live => self.live_config.as_ref().unwrap().cases.clone(),
            TradeMode::Backtest => self.backtest_config.as_ref().unwrap().cases.clone(),
            TradeMode::Simulated => self.simulate_config.as_ref().unwrap().cases.clone(),
        };
        
        // 遍历case
        for case in cases {
            let case_result = self.evaluate_case(case.clone()).await;
            // 如果为true，则发送消息到下一个节点, 并且后续的case不进行评估
            if case_result {
                let case_sender = self.get_output_handle().get(&format!("if_else_node_case_{}_output", case.case_id)).unwrap();
                // 节点信息
                let signal_message = SignalMessage {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: format!("if_else_node_case_{}_output", case.case_id),
                    signal_type: SignalType::ConditionMatch,
                    signal: Signal::True,
                    message_timestamp: get_utc8_timestamp()
                };

                // 获取case的handle
                let case_handle = self.get_output_handle().get(&format!("if_else_node_case_{}_output", case.case_id)).expect("case handle not found");
                tracing::debug!("{}：节点信息: {:?}", self.get_node_id(), signal_message);
                if case_handle.connect_count > 0 {
                    tracing::debug!("{}发送信号: {:?}", case_handle.output_handle_id, signal_message);
                    if let Err(e) = case_sender.message_sender.send(NodeMessage::Signal(signal_message.clone())) {
                        tracing::error!("节点 {} 发送信号失败: {}", self.get_node_id(), e);
                    }

                }
                

                // 发送事件
                if self.is_enable_event_publish().clone() {
                    let event = Event::Strategy(StrategyEvent::NodeMessage(NodeMessage::Signal(signal_message)));
                    if let Err(e) = self.get_event_publisher().publish(event.into()) {
                        tracing::error!(
                            node_id = %self.get_node_id(),
                            "条件节点发送信号事件失败"
                        );
                    }
                }
                case_matched = true;
                break;
            }
        }

        // 只有当所有case都为false时才执行else
        if !case_matched {
            let else_sender = self.get_output_handle().get("if_else_node_else_output").unwrap();
            let signal_message = SignalMessage {
                from_node_id: self.get_node_id().clone(),
                from_node_name: self.get_node_name().clone(),
                from_node_handle_id: self.get_output_handle().get("if_else_node_else_output").unwrap().output_handle_id.clone(),
                signal: Signal::True,
                signal_type: SignalType::ConditionMatch,
                message_timestamp: get_utc8_timestamp()
            };
            

            let else_handle = self.get_output_handle().get("if_else_node_else_output").expect("else handle not found");
            if else_handle.connect_count > 0 {
                // tracing::debug!("条件节点发送信号: {:?}", signal_message);
                if let Err(e) = else_sender.message_sender.send(NodeMessage::Signal(signal_message.clone())) {
                    tracing::error!("节点 {} 发送信号失败: {}", self.get_node_id(), e);
                }
            }

            // 发送事件
            if self.is_enable_event_publish().clone() {
                let event = Event::Strategy(StrategyEvent::NodeMessage(NodeMessage::Signal(signal_message)));
                if let Err(e) = self.get_event_publisher().publish(event.into()) {
                    tracing::error!(
                        node_id = %self.get_node_id(),
                        "条件节点发送信号事件失败"
                    );
                }
            }
        }
                        

    }

    pub async fn evaluate_case(&self, case: Case) -> bool {
        let logic_operator = case.logic_operator;
        match logic_operator {
            LogicOperator::And => {
                self.evaluate_and_conditions(case.conditions).await
            }
            LogicOperator::Or => {
                self.evaluate_or_conditions(case.conditions).await
            }
        }
    }

    fn get_variable_value(
        node_id: &str, 
        variable_name: &str, 
        received_value: &HashMap<String, Option<NodeMessage>>
    ) -> Option<f64> {
        received_value
            .get(node_id)?
            .as_ref()?
            .as_indicator()?
            .indicator_data
            .get_latest_indicator_value()
            .get(variable_name)
            .map(|v| v.value)
    }

    // 评估and条件组
    async fn evaluate_and_conditions(&self, conditions: Vec<Condition>) -> bool{
        let received_value = &self.received_value;
        
        for condition in conditions {
            // 获取左值
            let left_node_id = condition.left_variable.node_id.unwrap();
            let left_variabale = condition.left_variable.variable;
            let left_value = Self::get_variable_value(&left_node_id, &left_variabale, received_value);

            // 获取右值
            let right_var_type = condition.right_variable.var_type;
            let right_value = match right_var_type {
                VarType::Variable => {
                    let right_node_id = condition.right_variable.node_id.unwrap();
                    let right_variabale = condition.right_variable.variable;
                    Self::get_variable_value(&right_node_id, &right_variabale, received_value)
                },
                VarType::Constant => {
                    let right_variabale = condition.right_variable.variable;
                    Some(right_variabale.parse::<f64>().unwrap())
                },
            };

            let operator = condition.comparison_operator;
            
            if left_value.is_some() && right_value.is_some() {
                let left_value = left_value.unwrap();
                let right_value = right_value.unwrap();
                let condition_result = match operator {
                    ComparisonOperator::GreaterThan => left_value > right_value,
                    ComparisonOperator::LessThan => left_value < right_value,
                    ComparisonOperator::Equal => (left_value - right_value).abs() < f64::EPSILON, // 浮点数比较
                    ComparisonOperator::GreaterThanOrEqual => left_value >= right_value,
                    ComparisonOperator::LessThanOrEqual => left_value <= right_value,
                    ComparisonOperator::NotEqual => left_value != right_value,
                };
                // tracing::warn!("左值: {:?}, 比较符号: {:?}, 右值: {:?}, 结果: {:?}", left_value, operator.to_string(), right_value, condition_result);
                
                // 如果有任何一个条件不满足，立即返回false并中止后续条件判断
                if !condition_result {
                    return false;
                }
            }
        }
        // 所有条件都满足
        true
    }

    async fn evaluate_or_conditions(&self, conditions: Vec<Condition>) -> bool {
        let received_value = &self.received_value;
        
        if conditions.is_empty() {
            return false;
        }
        
        for condition in conditions {
            // 获取左值
            let left_node_id = condition.left_variable.node_id.unwrap();
            let left_variabale = condition.left_variable.variable;
            let left_value = Self::get_variable_value(&left_node_id, &left_variabale, received_value);

            // 获取右值
            let right_var_type = condition.right_variable.var_type;
            let right_value = match right_var_type {
                VarType::Variable => {
                    let right_node_id = condition.right_variable.node_id.unwrap();
                    let right_variabale = condition.right_variable.variable;
                    Self::get_variable_value(&right_node_id, &right_variabale, received_value)
                },
                VarType::Constant => {
                    let right_variabale = condition.right_variable.variable;
                    Some(right_variabale.parse::<f64>().unwrap())
                },
            };

            let operator = condition.comparison_operator;
            
            if left_value.is_some() && right_value.is_some() {
                let left_value = left_value.unwrap();
                let right_value = right_value.unwrap();
                let condition_result = match operator {
                    ComparisonOperator::GreaterThan => left_value > right_value,
                    ComparisonOperator::LessThan => left_value < right_value,
                    ComparisonOperator::Equal => (left_value - right_value).abs() < f64::EPSILON,
                    ComparisonOperator::GreaterThanOrEqual => left_value >= right_value,
                    ComparisonOperator::LessThanOrEqual => left_value <= right_value,
                    ComparisonOperator::NotEqual => left_value != right_value,
                };
                // tracing::warn!("左值: {:?}, 比较符号: {:?}, 右值: {:?}, 结果: {:?}", left_value, operator.to_string(), right_value, condition_result);
                
                // 如果有任何一个条件满足，立即返回true并中止后续条件判断
                if condition_result {
                    return true;
                }
            }
        }
        // 所有条件都不满足
        false
    }
}
