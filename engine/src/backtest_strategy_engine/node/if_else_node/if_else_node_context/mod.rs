mod context_impl;
mod event_handler;

use event_center::event::node_event::backtest_node_event::BacktestNodeEvent;
use event_center::event::node_event::backtest_node_event::common_event::{CommonEvent, TriggerEvent, TriggerPayload};
use event_center::event::node_event::backtest_node_event::if_else_node_event::{
    ConditionMatchEvent, ConditionMatchPayload, IfElseNodeEvent,
};
use std::collections::HashMap;
use std::fmt::Debug;

use super::if_else_node_type::IfElseNodeBacktestConfig;
use event_center::event::strategy_event::{StrategyRunningLogEvent, StrategyRunningLogType};
use tokio::sync::oneshot;

use super::condition::*;
use super::utils::get_condition_variable_value;
use crate::backtest_strategy_engine::node::node_context::{BacktestBaseNodeContext, BacktestNodeContextTrait};
use crate::backtest_strategy_engine::node::node_message::if_else_node_log_message::ConditionMatchedMsg;
use crate::backtest_strategy_engine::node::node_types::NodeOutputHandle;
use event_center::communication::Response;
use event_center::communication::backtest_strategy::GetCurrentTimeCommand;
use event_center::event::strategy_event::StrategyRunningLogSource;
use snafu::ResultExt;
use star_river_core::custom_type::NodeId;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::if_else_node_error::*;
use star_river_core::system::DateTimeUtc;

pub type ConfigId = i32;

#[derive(Debug, Clone)]
pub struct IfElseNodeContext {
    pub base_context: BacktestBaseNodeContext,
    pub received_flag: HashMap<(NodeId, ConfigId), bool>, // 用于记录每个variable的数据是否接收
    pub received_message: HashMap<(NodeId, ConfigId), Option<BacktestNodeEvent>>, // 用于记录每个variable的数据(node_id + variable_id)为key
    pub backtest_config: IfElseNodeBacktestConfig,
}

impl IfElseNodeContext {
    fn update_received_flag(&mut self, from_node_id: NodeId, from_variable_id: ConfigId, flag: bool) {
        self.received_flag
            .entry((from_node_id, from_variable_id))
            .and_modify(|e| *e = flag)
            .or_insert(flag);
    }

    // 初始化接收标记
    pub async fn init_received_data(&mut self) {
        for case in &self.backtest_config.cases {
            for condition in &case.conditions {
                // 处理左值
                if let (Some(left_node_id), Some(left_variable_id)) =
                    (condition.left_variable.node_id.clone(), condition.left_variable.variable_config_id)
                {
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
    pub async fn evaluate(&mut self) -> Result<(), IfElseNodeError> {
        let mut case_matched = false; // 是否匹配到case
        let current_time = self.get_current_time().await.unwrap();

        // 遍历case进行条件评估
        for (index, case) in self.backtest_config.cases.iter().enumerate() {
            let case_result = self.evaluate_case(case).await;

            // 如果条件匹配，处理匹配的case
            if case_result.0 {
                tracing::debug!("[{}] 条件匹配，处理匹配的case 分支", self.get_node_name());
                self.handle_matched_case(case, case_result.1, current_time).await?;
            }
            // 如果条件不匹配，并且是最后一个case, 则发送trigger事件
            else {
                if index == self.backtest_config.cases.len() - 1 {
                    tracing::debug!("[{}] 条件不匹配，发送trigger事件", self.get_node_name());
                    self.handle_not_matched_case(case).await;
                }
            }
            case_matched = true;
            break; // 找到匹配的case后立即退出
        }

        // 如果没有case匹配，处理else分支
        if !case_matched {
            tracing::debug!("[{}] 条件不匹配，处理else分支", self.get_node_name());
            self.handle_else_branch().await;
        }

        Ok(())
    }

    // 处理匹配的case
    async fn handle_matched_case(
        &self,
        case: &Case,
        condition_results: Vec<ConditionResult>,
        current_time: DateTimeUtc,
    ) -> Result<(), IfElseNodeError> {
        let strategy_id = self.get_strategy_id().clone();
        let from_node_id = self.get_node_id().clone();
        let from_node_name = self.get_node_name().clone();
        let play_index = self.get_play_index();

        let case_output_handle_id = format!("{}_output_{}", self.get_node_id(), case.case_id);
        let case_output_handle: &NodeOutputHandle = self.get_output_handle(&case_output_handle_id);
        let strategy_output_handle = self.get_strategy_output_handle();

        // 创建条件匹配事件
        let payload = ConditionMatchPayload::new(play_index);
        let condition_match_event: IfElseNodeEvent =
            ConditionMatchEvent::new(from_node_id.clone(), from_node_name.clone(), case_output_handle_id, payload).into();

        // 创建并发送日志事件
        let condition_result_json = serde_json::to_value(condition_results).context(EvaluateResultSerializationFailedSnafu {})?;
        let message = ConditionMatchedMsg::new(from_node_name.clone(), case.case_id);
        let log_event = StrategyRunningLogEvent::success(
            strategy_id,
            from_node_id.clone(),
            from_node_name.clone(),
            StrategyRunningLogSource::Node,
            StrategyRunningLogType::ConditionMatch,
            message.to_string(),
            condition_result_json,
            current_time,
        );
        let _ = strategy_output_handle.send(log_event.into());

        // 根据节点类型处理事件发送
        if self.is_leaf_node() {
            // 叶子节点：发送执行结束事件
            self.send_execute_over_event().await;
        } else {
            // 非叶子节点：将事件传递给下游节点
            let _ = case_output_handle.send(condition_match_event.into());
        }

        Ok(())
    }

    async fn handle_not_matched_case(&self, case: &Case) {
        if self.is_leaf_node() {
            self.send_execute_over_event().await;
            return;
        }

        let case_output_handle_id = format!("{}_output_{}", self.get_node_id(), case.case_id);
        let case_output_handle: &NodeOutputHandle = self.get_output_handle(&case_output_handle_id);
        let payload = TriggerPayload::new(self.get_play_index());

        let condition_not_match_event: CommonEvent = TriggerEvent::new(
            self.get_node_id().clone(),
            self.get_node_name().clone(),
            case_output_handle_id.clone(),
            payload,
        )
        .into();
        let _ = case_output_handle.send(condition_not_match_event.into());
    }

    // 处理else分支
    async fn handle_else_branch(&self) {
        if self.is_leaf_node() {
            // 叶子节点：发送执行结束事件
            self.send_execute_over_event().await;
        } else {
            // 非叶子节点：发送事件到else输出句柄
            let else_output_handle = self.get_default_output_handle();
            let payload = ConditionMatchPayload::new(self.get_play_index());
            let else_event: IfElseNodeEvent = ConditionMatchEvent::new(
                self.get_node_id().clone(),
                self.get_node_name().clone(),
                else_output_handle.output_handle_id.clone(),
                payload,
            )
            .into();
            let _ = else_output_handle.send(else_event.into());
        }
    }

    // 发送执行结束事件的公共方法
    // async fn send_execute_over_event(&self) {
    //     let payload = ExecuteOverPayload::new(self.get_play_index());
    //     let execute_over_event: CommonEvent = ExecuteOverEvent::new(
    //         self.get_node_id().clone(),
    //         self.get_node_name().clone(),
    //         self.get_node_id().clone(),
    //         payload,
    //     )
    //     .into();
    //     let strategy_output_handle = self.get_strategy_output_handle();
    //     let _ = strategy_output_handle.send(execute_over_event.into());
    // }

    pub async fn evaluate_case(&self, case: &Case) -> (bool, Vec<ConditionResult>) {
        match case.logical_symbol {
            LogicalSymbol::And => self.evaluate_and_conditions(&case.conditions).await,
            LogicalSymbol::Or => self.evaluate_or_conditions(&case.conditions).await,
        }
    }

    // 评估单个条件
    fn evaluate_single_condition(&self, condition: &Condition) -> ConditionResult {
        let received_value = &self.received_message;

        // 获取左值
        let left_value = get_condition_variable_value(&condition.left_variable, received_value);

        // 获取右值
        let right_value = get_condition_variable_value(&condition.right_variable, received_value);

        // 获取符号
        let comparison_symbol = &condition.comparison_symbol;

        if let (Some(left_value), Some(right_value)) = (&left_value, &right_value) {
            let condition_result = match comparison_symbol {
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
                comparison_symbol.to_string(),
                condition.right_variable.variable,
                right_value,
                condition_result
            );

            ConditionResult {
                condition_id: condition.condition_id,
                left_variable: condition.left_variable.clone(),
                right_variable: condition.right_variable.clone(),
                comparison_symbol: condition.comparison_symbol.clone(),
                left_value: Some(*left_value),
                right_value: Some(*right_value),
                condition_result: condition_result,
            }
        } else {
            tracing::warn!(
                "条件评估失败: 左值={:?}, 右值={:?}, 存在空值, 当前play_index: {}",
                left_value,
                right_value,
                self.get_play_index()
            );
            ConditionResult {
                condition_id: condition.condition_id,
                left_variable: condition.left_variable.clone(),
                right_variable: condition.right_variable.clone(),
                comparison_symbol: comparison_symbol.clone(),
                left_value,
                right_value,
                condition_result: false,
            }
        }
    }

    // 评估and条件组
    async fn evaluate_and_conditions(&self, conditions: &Vec<Condition>) -> (bool, Vec<ConditionResult>) {
        // tracing::debug!("{}: 开始评估and条件组: {:#?}", self.get_node_id(), conditions);
        if conditions.is_empty() {
            return (true, vec![]); // 空条件组默认为true
        }

        let mut condition_results = vec![];
        // 使用迭代器的all方法，更简洁且在第一个false时短路
        let result = conditions.iter().all(|condition| {
            let condition_result = self.evaluate_single_condition(condition);
            let result = condition_result.condition_result.clone();
            condition_results.push(condition_result);
            result
        });
        (result, condition_results)
    }

    // 评估or条件组
    async fn evaluate_or_conditions(&self, conditions: &Vec<Condition>) -> (bool, Vec<ConditionResult>) {
        if conditions.is_empty() {
            return (false, vec![]); // 空条件组默认为false
        }

        let mut condition_results = vec![];
        // 使用迭代器的any方法，更简洁且在第一个true时短路
        let result = conditions.iter().any(|condition| {
            let condition_result = self.evaluate_single_condition(condition);
            let result = condition_result.condition_result.clone();
            condition_results.push(condition_result);
            result
        });
        (result, condition_results)
    }

    async fn get_current_time(&self) -> Result<DateTimeUtc, String> {
        let (tx, rx) = oneshot::channel();

        let cmd = GetCurrentTimeCommand::new(self.get_node_id().clone(), tx, None);
        self.get_strategy_command_sender().send(cmd.into()).await.unwrap();

        let response = rx.await.unwrap();
        if response.is_success() {
            return Ok(response.current_time.clone());
        } else {
            return Err(response.get_error().to_string());
        }
    }
}
