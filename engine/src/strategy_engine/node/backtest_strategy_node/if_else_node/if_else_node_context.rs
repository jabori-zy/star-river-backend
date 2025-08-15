use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp;
use utils::get_utc8_timestamp_millis;
use event_center::Event;
use types::strategy::node_event::{SignalEvent, BacktestNodeEvent, BacktestConditionMatchEvent, IndicatorNodeEvent, PlayIndexUpdateEvent};
use super::if_else_node_type::IfElseNodeBacktestConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use super::condition::*;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use types::custom_type::{NodeId, HandleId, VariableId};
use super::utils::{get_variable_value, get_condition_variable_value};
use types::strategy::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::custom_type::PlayIndex;

#[derive(Debug, Clone)]
pub struct IfElseNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub is_processing: bool,
    pub received_flag: HashMap<(NodeId, VariableId), bool>, // 用于记录每个variable的数据是否接收
    pub received_message: HashMap<(NodeId, VariableId), Option<BacktestNodeEvent>>, // 用于记录每个variable的数据(node_id + variable_id)为key
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
        let default_output_handle_id = format!("{}_else_output", self.get_node_id());
        self.base_context.output_handles.get(&default_output_handle_id).unwrap().clone()
    }


    async fn handle_event(&mut self, event: Event) -> Result<(), String> {
        let _event = event;
        Ok(())
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) -> Result<(), String> {
        // tracing::debug!("{}: 收到节点事件: {:?}", self.get_node_id(), node_event);
        //如果事件类型是回测指标更新或者k线更新
        match &node_event {
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(backtest_indicator_update_event)) => {
                // 如果回测指标更新事件的k线缓存索引与播放索引相同，则更新接收事件
                if self.get_play_index() == backtest_indicator_update_event.play_index {
                    self.update_received_event(node_event);
                }
            }
            BacktestNodeEvent::KlineNode(kline_event) => {
                // 如果回测k线更新事件的k线缓存索引与播放索引相同，则更新接收事件
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    if self.get_play_index() == kline_update_event.play_index {
                        self.update_received_event(node_event);
                    }
                }
            }
            BacktestNodeEvent::Variable(variable_message) => {
                self.update_received_event(node_event);
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) -> Result<(), String> {
        match strategy_inner_event {
            // StrategyInnerEvent::PlayIndexUpdate(play_index_update_event) => {
            //     // 更新播放索引
            //     self.set_play_index(play_index_update_event.play_index).await;
            //     let strategy_output_handle_id = format!("{}_strategy_output", self.get_node_id());
            //     let signal = BacktestNodeEvent::Signal(SignalEvent::PlayIndexUpdated(PlayIndexUpdateEvent {
            //         from_node_id: self.get_node_id().clone(),
            //         from_node_name: self.get_node_name().clone(),
            //         from_node_handle_id: strategy_output_handle_id.clone(),
            //         play_index: self.get_play_index().await,
            //         message_timestamp: get_utc8_timestamp_millis(),
            //     }));
            //     self.get_strategy_output_handle().send(signal).unwrap();
            // }
            StrategyInnerEvent::NodeReset => {
                tracing::info!("{}: 收到节点重置事件", self.base_context.node_id);
            }
        }
        Ok(())
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) -> Result<(), String> {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
        Ok(())
    }


}

impl IfElseNodeContext {

    fn update_received_event(&mut self, received_event: BacktestNodeEvent) {
        // tracing::debug!("接收到的变量消息: {:?}", received_event);
        let (from_node_id, from_variable_id) = match &received_event {
            BacktestNodeEvent::IndicatorNode(indicator_message) => {
                if let IndicatorNodeEvent::IndicatorUpdate(indicator_update_event) = indicator_message {
                    let from_node_id = indicator_update_event.from_node_id.clone();
                    // let from_handle_id = indicator_update_event.from_handle_id.clone();
                    let from_variable_id = indicator_update_event.indicator_id;
                    (from_node_id, from_variable_id)
                } else {
                    return;
                }
            }
            // NodeEvent::Variable(variable_message) => {
            //     variable_message.from_node_id.clone()
            // }
            _ => {
                return;
            }
        };
        self.received_message.entry((from_node_id.clone(), from_variable_id))
        .and_modify(|e| *e = Some(received_event.clone()))
        .or_insert(Some(received_event));
        // tracing::debug!("received_message: {:?}", self.received_message);
        
        self.update_received_flag(from_node_id, from_variable_id, true);
    }

    fn update_received_flag(&mut self, from_node_id: NodeId, from_variable_id: VariableId, flag: bool) {
        self.received_flag.entry((from_node_id, from_variable_id))
        .and_modify(|e| *e = flag)
        .or_insert(flag);
    }

    // 初始化接收标记
    pub async fn init_received_data(&mut self) {
        for case in &self.backtest_config.cases {
            for condition in &case.conditions {
                // 处理左值
                if let (Some(left_node_id), Some(left_variable_id)) = (
                    condition.left_variable.node_id.clone(),
                    condition.left_variable.variable_id,
                ) {
                    let key = (left_node_id, left_variable_id);
                    self.received_flag.insert(key.clone(), false);
                    self.received_message.insert(key, None);
                }

                // 处理右值（如果是变量类型）
                if matches!(condition.right_variable.var_type, VarType::Variable) {
                    if let (Some(right_node_id), Some(right_variable_id)) = (
                        condition.right_variable.node_id.clone(),
                        condition.right_variable.variable_id,
                    ) {
                        let key = (right_node_id, right_variable_id);
                        self.received_flag.insert(key.clone(), false);
                        self.received_message.insert(key, None);
                    }
                }
            }
        }
        tracing::debug!(node_id = %self.get_node_id(), "init received data success: {:?}, {:?}", self.received_flag, self.received_message);
    }

    pub fn is_all_value_received(&self) -> bool {
        // 直接检查所有标志是否都为true
        self.received_flag.values().all(|&flag| flag)
    }

    pub fn reset_received_flag(&mut self) {
        for flag in self.received_flag.values_mut() {
            *flag = false;
        }
    }

    // 开始评估各个分支
    pub async fn evaluate(&mut self) {
        // 在锁外执行评估
        let mut case_matched = false; // 是否匹配到case
        // 根据交易模式获取case

        
        // 遍历case
        for case in self.backtest_config.cases.iter() {
            let case_result = self.evaluate_case(case).await;
            // tracing::debug!("{}: case_result: {:?}", self.get_node_id(), case_result);

            // 如果为true，则发送消息到下一个节点, 并且后续的case不进行评估
            if case_result {
                // 节点信息
                let case_output_handle_id = format!("{}_output{}", self.get_node_id(), case.case_id);
                let case_output_handle = self.get_output_handle(&case_output_handle_id);
                let signal_event = SignalEvent::BacktestConditionMatch(BacktestConditionMatchEvent {
                    from_node_id: self.get_node_id().clone(),
                    from_node_name: self.get_node_name().clone(),
                    from_node_handle_id: case_output_handle_id.clone(),
                    play_index: self.get_play_index(),
                    message_timestamp: get_utc8_timestamp()
                });
                // tracing::debug!("{}: 信号消息: {:?}", self.get_node_name(), signal_event);

                // 获取case的handle
                // tracing::debug!("{}：节点发送信号事件: {:?}", self.get_node_id(), signal_event);
                if let Err(e) = case_output_handle.send(BacktestNodeEvent::Signal(signal_event.clone())) {
                    // tracing::error!("{}: 发送信号事件失败: {:?}", self.get_node_id(), e);
                }

                case_matched = true;
                break;
            }
        }

        // 只有当所有case都为false时才执行else
        let default_ouput_handle = self.get_default_output_handle(); // 获取else的输出句柄,else是默认出口
        if !case_matched {
            let signal_event = SignalEvent::BacktestConditionMatch(BacktestConditionMatchEvent {
                from_node_id: self.get_node_id().clone(),
                from_node_name: self.get_node_name().clone(),
                from_node_handle_id: default_ouput_handle.output_handle_id.clone(),
                play_index: self.get_play_index(),
                message_timestamp: get_utc8_timestamp()
            });
            

            // tracing::debug!("{}: 发送信号事件: {:?}", self.get_node_id(), signal_event);
            if let Err(e) = default_ouput_handle.send(BacktestNodeEvent::Signal(signal_event.clone())) {
                // tracing::error!("{}: 发送信号事件失败: {:?}", self.get_node_id(), e);
            }
        }
                        

    }

    pub async fn evaluate_case(&self, case: &Case) -> bool {
        match case.logical_symbol {
            LogicalSymbol::And => {
                self.evaluate_and_conditions(&case.conditions).await
            }
            LogicalSymbol::Or => {
                self.evaluate_or_conditions(&case.conditions).await
            }
        }
    }

    
    

    

    // 评估单个条件
    fn evaluate_single_condition(&self, condition: &Condition) -> bool {
        let received_value = &self.received_message;

        // 获取左值
        let left_value = get_condition_variable_value(&condition.left_variable, received_value);
        
        // 获取右值
        let right_value = get_condition_variable_value(&condition.right_variable, received_value);

        if let (Some(left_value), Some(right_value)) = (left_value, right_value) {
            let condition_result = match condition.comparison_symbol {
                ComparisonSymbol::GreaterThan => left_value > right_value,
                ComparisonSymbol::LessThan => left_value < right_value,
                ComparisonSymbol::Equal => (left_value - right_value).abs() < f64::EPSILON, // 浮点数比较
                ComparisonSymbol::GreaterThanOrEqual => left_value >= right_value,
                ComparisonSymbol::LessThanOrEqual => left_value <= right_value,
                ComparisonSymbol::NotEqual => (left_value - right_value).abs() >= f64::EPSILON,
            };
            
            // tracing::debug!(
            //     "条件评估: 左值={:.6}, 比较符号={}, 右值={:.6}, 结果={}",
            //     left_value,
            //     condition.comparison_symbol.to_string(),
            //     right_value,
            //     condition_result
            // );
            
            condition_result
        } else {
            tracing::debug!(
                "条件评估失败: 左值={:?}, 右值={:?}, 存在空值",
                left_value,
                right_value
            );
            false
        }
    }

    // 评估and条件组
    async fn evaluate_and_conditions(&self, conditions: &Vec<Condition>) -> bool {
        if conditions.is_empty() {
            return true; // 空条件组默认为true
        }

        // 使用迭代器的all方法，更简洁且在第一个false时短路
        conditions.iter().all(|condition| self.evaluate_single_condition(condition))
    }

    // 评估or条件组
    async fn evaluate_or_conditions(&self, conditions: &Vec<Condition>) -> bool {
        if conditions.is_empty() {
            return false; // 空条件组默认为false
        }

        // 使用迭代器的any方法，更简洁且在第一个true时短路
        conditions.iter().any(|condition| self.evaluate_single_condition(condition))
    }
}
