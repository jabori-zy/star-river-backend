use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp;
use utils::get_utc8_timestamp_millis;
use event_center::strategy_event::StrategyEvent;
use event_center::Event;
use types::strategy::node_event::{SignalEvent, NodeEvent, BacktestConditionMatchEvent, IndicatorEvent, PlayIndexUpdateEvent};
use super::if_else_node_type::IfElseNodeBacktestConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use super::condition::*;
use types::strategy::strategy_inner_event::StrategyInnerEvent;

#[derive(Debug, Clone)]
pub struct IfElseNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub is_processing: bool,
    pub received_flag: HashMap<String, bool>, // 用于记录每个节点的数据是否接收完成
    pub received_message: HashMap<String, Option<NodeEvent>>, // 用于记录每个节点的数据
    pub backtest_config: IfElseNodeBacktestConfig,
    

}



#[async_trait]
impl BacktestNodeContextTrait for IfElseNodeContext {
    
    fn clone_box(&self) -> Box<dyn BacktestNodeContextTrait> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn get_base_context(&self) -> &BacktestBaseNodeContext {
        &self.base_context
    }

    fn get_base_context_mut(&mut self) -> &mut BacktestBaseNodeContext {
        &mut self.base_context
    }

    fn get_default_output_handle(&self) -> NodeOutputHandle {
        self.base_context.output_handle.get(&format!("if_else_node_else_output")).unwrap().clone()
    }


    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let _event = event;
        Ok(())
    }

    async fn handle_node_event(&mut self, node_event: NodeEvent) -> Result<(), String> {
        tracing::debug!("{}: 收到节点事件: {:?}", self.get_node_id(), node_event);
        //如果事件类型是回测指标更新或者k线更新
        match &node_event {
            NodeEvent::Indicator(IndicatorEvent::BacktestIndicatorUpdate(backtest_indicator_update_event)) => {
                // 如果回测指标更新事件的k线缓存索引与播放索引相同，则更新接收事件
                if self.get_play_index().await == backtest_indicator_update_event.kline_cache_index {
                    self.update_received_event(node_event);
                }
            }
            NodeEvent::BacktestKline(backtest_kline_update_event) => {
                // 如果回测k线更新事件的k线缓存索引与播放索引相同，则更新接收事件
                if self.get_play_index().await == backtest_kline_update_event.kline_cache_index {
                    self.update_received_event(node_event);
                }
            }
            NodeEvent::Variable(variable_message) => {
                self.update_received_event(node_event);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
                // 更新播放索引
                self.set_play_index(play_index_update_event.played_index).await;
                // tracing::debug!("{}: 更新播放索引: {}", self.get_node_id(), play_index_update_event.played_index);
                let signal = NodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: self.get_default_output_handle().output_handle_id.clone(),
                    node_play_index: self.get_play_index().await,
                    message_timestamp: get_utc8_timestamp_millis(),
                }));
                self.get_default_output_handle().send(signal).unwrap();
            }
        }
        Ok(())
    }
}

impl IfElseNodeContext {

    fn update_received_event(&mut self, received_event: NodeEvent) {
        tracing::debug!("接收到的变量消息: {:?}", received_event);
        let from_node_id = match &received_event {
            NodeEvent::Indicator(indicator_message) => {
                if let IndicatorEvent::BacktestIndicatorUpdate(indicator_update_event) = indicator_message {
                    indicator_update_event.from_node_id.clone()
                } else {
                    return;
                }
            }
            NodeEvent::Variable(variable_message) => {
                variable_message.from_node_id.clone()
            }
            _ => {
                return;
            }
        };
        self.received_message.entry(from_node_id.clone())
        .and_modify(|e| *e = Some(received_event.clone()))
        .or_insert(Some(received_event));
        
        self.update_received_flag(from_node_id, true);
    }

    fn update_received_flag(&mut self, from_node_id: String, flag: bool) {
        self.received_flag.entry(from_node_id.clone())
        .and_modify(|e| *e = flag)
        .or_insert(flag);
    }

    // 初始化接收标记
    pub async fn init_received_flag(&mut self) {
        for from_node_id in self.get_from_node_id().clone() {
            self.received_flag.insert(from_node_id.clone(), false);
        }
        
    }

    pub async fn init_received_value(&mut self) {
        for from_node_id in self.get_from_node_id().clone() {
            self.received_message.insert(from_node_id.clone(), None);
        }
    }



    // 开始评估各个分支
    pub async fn evaluate(&mut self) {
        // 在锁外执行评估
        let mut case_matched = false; // 是否匹配到case
        // 根据交易模式获取case

        
        // 遍历case
        for case in self.backtest_config.cases.clone() {
            let case_result = self.evaluate_case(case.clone()).await;
            // tracing::debug!("{}: case_result: {:?}", self.get_node_id(), case_result);

            // 如果为true，则发送消息到下一个节点, 并且后续的case不进行评估
            if case_result {
                // 节点信息
                let signal_event = SignalEvent::BacktestConditionMatch(BacktestConditionMatchEvent {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: format!("if_else_node_case_{}_output", case.case_id),
                    play_index: self.get_play_index().await,
                    message_timestamp: get_utc8_timestamp()
                });
                // tracing::debug!("{}: 信号消息: {:?}", self.get_node_name(), signal_event);

                // 获取case的handle
                let case_handle = self.get_all_output_handle().get(&format!("if_else_node_case_{}_output", case.case_id)).expect("case handle not found");
                tracing::debug!("{}：节点发送信号事件: {:?}", self.get_node_id(), signal_event);
                case_handle.send(NodeEvent::Signal(signal_event.clone())).unwrap();

                // 发送事件
                if self.is_enable_event_publish().clone() {
                    let event = Event::Strategy(StrategyEvent::NodeMessageUpdate(NodeEvent::Signal(signal_event.clone())));
                    if let Err(e) = self.get_event_publisher().publish(event.into()).await {
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
            let signal_event = SignalEvent::BacktestConditionMatch(BacktestConditionMatchEvent {
                from_node_id: self.get_node_id().clone(),
                from_node_name: self.get_node_name().clone(),
                from_node_handle_id: self.get_all_output_handle().get("if_else_node_else_output").unwrap().output_handle_id.clone(),
                play_index: self.get_play_index().await,
                message_timestamp: get_utc8_timestamp()
            });
            

            let else_handle = self.get_all_output_handle().get("if_else_node_else_output").expect("else handle not found");
            tracing::debug!("{}: 发送信号事件: {:?}", self.get_node_id(), signal_event);
            else_handle.send(NodeEvent::Signal(signal_event.clone())).unwrap();


            // 发送事件
            if self.is_enable_event_publish().clone() {
                let event = Event::Strategy(StrategyEvent::NodeMessageUpdate(NodeEvent::Signal(signal_event.clone())));
                if let Err(e) = self.get_event_publisher().publish(event.into()).await {
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

    // 获取变量值
    fn get_variable_value(
        node_id: &str, 
        variable_name: &str,
        received_value: &HashMap<String, Option<NodeEvent>>
    ) -> Option<f64> {
        let node_event = received_value.get(node_id)?.as_ref()?;
        
        match node_event {
            NodeEvent::Indicator(indicator_event) => {
                if let IndicatorEvent::BacktestIndicatorUpdate(indicator_update_event) = indicator_event {
                    indicator_update_event
                        .indicator_series
                        .last()
                        .and_then(|last_indicator| {
                            let indicator_json = last_indicator.to_json();
                            indicator_json.get(variable_name).cloned()
                        })
                        .and_then(|indicator_value| {
                            indicator_value.as_f64().or_else(|| {
                                tracing::warn!("variable '{}'s value '{}' is not a number", variable_name, indicator_value);
                                None
                            })
                        })
                } else {
                    None
                }
            }
            NodeEvent::Variable(variable_message) => {
                Some(variable_message.variable_value)
            }
            _ => None
        }
    }

    // 评估and条件组
    async fn evaluate_and_conditions(&self, conditions: Vec<Condition>) -> bool{
        let received_value = &self.received_message;
        
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
                tracing::debug!("左值: {:?}, 比较符号: {:?}, 右值: {:?}, 结果: {:?}", left_value, operator.to_string(), right_value, condition_result);
                
                // 如果有任何一个条件不满足，立即返回false并中止后续条件判断
                if !condition_result {
                    return false;
                }
            }
            else {
                tracing::debug!("left value is: {:?}, right value is: {:?}, there is exist none value, condition is not match", left_value, right_value);
                return false;
            }
        }
        // 所有条件都满足
        true
    }

    async fn evaluate_or_conditions(&self, conditions: Vec<Condition>) -> bool {
        let received_value = &self.received_message;
        
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
            else {
                // tracing::warn!("left value is: {:?}, right value is: {:?}, there is exist none value, condition is not match", left_value, right_value);
                return false;
            }
        }
        // 所有条件都不满足
        false
    }
}
