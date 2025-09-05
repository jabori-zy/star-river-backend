use std::collections::HashMap;
use std::fmt::Debug;
use std::any::Any;
use async_trait::async_trait;
use utils::get_utc8_timestamp;
use event_center::Event;
use types::strategy::node_event::{SignalEvent, BacktestNodeEvent, BacktestConditionMatchEvent, IndicatorNodeEvent, BacktestConditionNotMatchEvent};
use super::if_else_node_type::IfElseNodeBacktestConfig;
use crate::strategy_engine::node::node_types::NodeOutputHandle;
use crate::strategy_engine::node::node_context::{BacktestBaseNodeContext,BacktestNodeContextTrait};
use super::condition::*;
use types::strategy::strategy_inner_event::StrategyInnerEvent;
use types::custom_type::NodeId;
use super::utils::{get_condition_variable_value};
use types::strategy::node_event::backtest_node_event::kline_node_event::KlineNodeEvent;
use event_center::command::backtest_strategy_command::StrategyCommand;
use types::strategy::node_event::backtest_node_event::variable_node_event::VariableNodeEvent;

pub type ConfigId = i32;


#[derive(Debug, Clone)]
pub struct IfElseNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub received_flag: HashMap<(NodeId, ConfigId), bool>, // 用于记录每个variable的数据是否接收
    pub received_message: HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>, // 用于记录每个variable的数据(node_id + variable_id)为key
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
        let else_output_handle_id = format!("{}_else_output", self.get_node_id());
        self.base_context.output_handles.get(&else_output_handle_id).unwrap().clone()
    }


    async fn handle_event(&mut self, event: Event) {
        let _event = event;
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        // tracing::debug!("{}: 收到节点事件: {:?}", self.get_node_id(), node_event);
        //如果事件类型是回测指标更新或者k线更新
        match &node_event {
            BacktestNodeEvent::IndicatorNode(IndicatorNodeEvent::IndicatorUpdate(indicator_update_event)) => {
                // 如果回测指标更新事件的k线缓存索引与播放索引相同，则更新接收事件
                if self.get_play_index() == indicator_update_event.play_index {
                    tracing::debug!("{}: 接收到指标更新事件。事件的play_index: {}，节点的play_index: {}", self.base_context.node_id, indicator_update_event.play_index, self.get_play_index());
                    self.update_received_event(node_event);
                }
            }
            BacktestNodeEvent::KlineNode(kline_event) => {
                // 如果回测k线更新事件的k线缓存索引与播放索引相同，则更新接收事件
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    if self.get_play_index() == kline_update_event.play_index {
                        tracing::debug!("{}: 接收到k线更新事件。事件的play_index: {}，节点的play_index: {}", self.base_context.node_id, kline_update_event.play_index, self.get_play_index());
                        self.update_received_event(node_event);
                    }
                }
            }
            BacktestNodeEvent::Variable(variable_event) => {
                let VariableNodeEvent::SysVariableUpdated(sys_variable_updated_event) = variable_event;
                tracing::debug!("{}: 收到变量更新事件。 事件的play_index: {}，节点的play_index: {}", self.get_node_id(), sys_variable_updated_event.play_index, self.get_play_index());
                self.update_received_event(node_event);
            }
            BacktestNodeEvent::Signal(signal_event) => {
                match signal_event {
                    SignalEvent::BacktestConditionNotMatch(_) => {
                        tracing::debug!("{}: 接收到条件不匹配事件。 不需要逻辑判断", self.get_node_id());

                        let all_output_handles = self.get_all_output_handles();
                        for (handle_id, handle) in all_output_handles.iter() {
                            if handle_id == &format!("{}_strategy_output", self.get_node_id()) {
                                continue;
                            }

                            if handle.connect_count > 0 {
                                let condition_not_match_event = SignalEvent::BacktestConditionNotMatch(BacktestConditionNotMatchEvent {
                                    from_node_id: self.get_node_id().clone(),
                                    from_node_name: self.get_node_name().clone(),
                                    from_node_handle_id: handle_id.clone(),
                                    play_index: self.get_play_index(),
                                    timestamp: get_utc8_timestamp()
                                });
                                
                                let _ = handle.send(BacktestNodeEvent::Signal(condition_not_match_event));
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    async fn handle_strategy_inner_event(&mut self, strategy_inner_event: StrategyInnerEvent) {
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
                // tracing::info!("{}: 收到节点重置事件", self.base_context.node_id);
            }
        }
        
    }

    async fn handle_strategy_command(&mut self, strategy_command: StrategyCommand) {
        // tracing::info!("{}: 收到策略命令: {:?}", self.base_context.node_id, strategy_command);
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
                    let from_variable_id = indicator_update_event.config_id;
                    (from_node_id, from_variable_id)
                } else {
                    return;
                }
            }
            BacktestNodeEvent::Variable(variable_event) => {
                let VariableNodeEvent::SysVariableUpdated(sys_variable_updated_event) = variable_event;
                (sys_variable_updated_event.from_node_id.clone(), sys_variable_updated_event.variable_config_id)
            }
            BacktestNodeEvent::KlineNode(kline_event) => {
                if let KlineNodeEvent::KlineUpdate(kline_update_event) = kline_event {
                    (kline_update_event.from_node_id.clone(), kline_update_event.config_id)
                } else {
                    return;
                }
            }
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

    fn update_received_flag(&mut self, from_node_id: NodeId, from_variable_id: ConfigId, flag: bool) {
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
                    condition.left_variable.variable_config_id,
                ) {
                    let key = (left_node_id, left_variable_id);
                    self.received_flag.insert(key.clone(), false);
                    self.received_message.insert(key, None);
                }

                // 处理右值（如果是变量类型）
                if matches!(condition.right_variable.var_type, VarType::Variable) {
                    if let (Some(right_node_id), Some(right_variable_id)) = (
                        condition.right_variable.node_id.clone(),
                        condition.right_variable.variable_config_id,
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
            let case_output_handle_id = format!("{}_output_{}", self.get_node_id(), case.case_id);
            let case_output_handle = self.get_output_handle(&case_output_handle_id);

            // 如果为true，则发送消息到下一个节点, 并且后续的case不进行评估
            let signal_event = {
                let from_node_id = self.get_node_id().clone();
                let from_node_name = self.get_node_name().clone();
                let play_index = self.get_play_index();
                let timestamp = get_utc8_timestamp();
                if case_result {
                    SignalEvent::BacktestConditionMatch(BacktestConditionMatchEvent {
                        from_node_id,
                        from_node_name,
                        from_node_handle_id: case_output_handle_id.clone(),
                        play_index,
                        timestamp
                    })
                } else {
                    SignalEvent::BacktestConditionNotMatch(BacktestConditionNotMatchEvent {
                        from_node_id,
                        from_node_name,
                        from_node_handle_id: case_output_handle_id.clone(),
                        play_index,
                        timestamp
                    })
                }
            };
            
            let _ = case_output_handle.send(BacktestNodeEvent::Signal(signal_event.clone()));
            case_matched = true;
            break;
        }

        // 只有当所有case都为false时才执行else
        let else_output_handle = self.get_default_output_handle(); // 获取else的输出句柄
        if !case_matched {
            let signal_event = SignalEvent::BacktestConditionMatch(BacktestConditionMatchEvent {
                from_node_id: self.get_node_id().clone(),
                from_node_name: self.get_node_name().clone(),
                from_node_handle_id: else_output_handle.output_handle_id.clone(),
                play_index: self.get_play_index(),
                timestamp: get_utc8_timestamp()
            });
            

            // tracing::debug!("{}: 发送信号事件: {:?}", self.get_node_id(), signal_event);
            if let Err(e) = else_output_handle.send(BacktestNodeEvent::Signal(signal_event.clone())) {
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
        tracing::debug!("{}: 开始评估case: 左变量名：{:?}, 右变量名：{:?}, 符号:{:?}, 当前的play_index: {}", self.get_node_id(), condition.left_variable.variable, condition.right_variable.variable, condition.comparison_symbol, self.get_play_index());
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
            
            tracing::debug!(
                "当前play_index: {}, 左变量名：{:#?}, 左值={:.6}, 比较符号:{:?}, 右变量名：{:#?}, 右值={:.6}, 结果={}",
                self.get_play_index(),
                condition.left_variable.variable,
                left_value,
                condition.comparison_symbol.to_string(),
                condition.right_variable.variable,
                right_value,
                condition_result
            );
            
            condition_result
        } else {
            tracing::error!(
                "条件评估失败: 左值={:?}, 右值={:?}, 存在空值, 当前play_index: {}",
                left_value,
                right_value,
                self.get_play_index()
            );
            false
        }
    }

    // 评估and条件组
    async fn evaluate_and_conditions(&self, conditions: &Vec<Condition>) -> bool {
        // tracing::debug!("{}: 开始评估and条件组: {:#?}", self.get_node_id(), conditions);
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
