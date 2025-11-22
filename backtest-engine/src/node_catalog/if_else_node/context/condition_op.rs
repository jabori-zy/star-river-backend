// External crate imports
use snafu::ResultExt;
// Current crate imports - star_river_core
use star_river_core::{custom_type::NodeId, system::DateTimeUtc};
// Current crate imports - star_river_event
use star_river_event::backtest_strategy::node_event::{
    IfElseNodeEvent,
    if_else_node_event::{
        CaseFalseEvent, CaseFalsePayload, CaseTrueEvent, CaseTruePayload, ElseFalseEvent, ElseFalsePayload, ElseTrueEvent, ElseTruePayload,
    },
};
// Current crate imports - strategy_core
use strategy_core::{
    benchmark::node_benchmark::CycleTracker,
    communication::strategy::StrategyResponse,
    event::{
        node_common_event::CommonEvent,
        strategy_event::{StrategyRunningLogEvent, StrategyRunningLogSource, StrategyRunningLogType},
    },
    node::context_trait::{NodeBenchmarkExt, NodeCommunicationExt, NodeHandleExt, NodeIdentityExt, NodeRelationExt},
    node_infra::if_else_node::{Case, Condition, ConditionResult, FormulaRight, LogicalSymbol},
};
use tokio::sync::oneshot;

// Relative imports
use super::{ConfigId, IfElseNodeContext};
// Local module imports
use crate::{
    node::{
        node_error::{IfElseNodeError, if_else_node_error::EvaluateResultSerializationFailedSnafu},
        node_message::if_else_node_log_message::ConditionMatchedMsg,
    },
    node_catalog::if_else_node::utils::{compare, parse_condition_left_value, parse_condition_right_value},
    strategy::strategy_command::{GetCurrentTimeCmdPayload, GetCurrentTimeCommand},
};

impl IfElseNodeContext {
    pub fn update_received_flag(&mut self, from_node_id: NodeId, from_variable_id: ConfigId, flag: bool) {
        self.received_flag
            .entry((from_node_id, from_variable_id))
            .and_modify(|e| *e = flag)
            .or_insert(flag);
    }

    // 初始化接收标记
    pub async fn init_received_data(&mut self) {
        for case in &self.node_config.cases {
            for condition in &case.conditions {
                // 处理左值

                let key = (condition.left.node_id.clone(), condition.left.var_config_id);
                self.received_flag.insert(key.clone(), false);
                self.received_message.insert(key, None);

                // 处理右值（如果是变量类型）
                if let FormulaRight::Variable(variable) = &condition.right {
                    let key = (variable.node_id.clone(), variable.var_config_id);
                    self.received_flag.insert(key.clone(), false);
                    self.received_message.insert(key, None);
                }
            }
        }
        tracing::debug!(node_id = %self.node_id(), "init received data success: {:?}, {:?}", self.received_flag, self.received_message);
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

    // Start evaluating all branches
    pub async fn evaluate(&mut self) -> Result<(), IfElseNodeError> {
        let mut cycle_tracker: CycleTracker = CycleTracker::new(self.play_index() as u32);

        let mut have_true_case = false; // Track whether any case has matched
        let current_time = self.get_current_time().await.unwrap();

        // Iterate through all cases for condition evaluation
        for case in self.node_config.cases.iter() {
            let phase_name = format!("evaluate case {}", case.case_id);
            cycle_tracker.start_phase(&phase_name);

            // Only evaluate condition if no case has matched yet
            if !have_true_case {
                // evaluate result (is true, result)
                let case_result = self.evaluate_case(case).await;

                // If condition matches, handle the matched case
                if case_result.0 {
                    // tracing::debug!("[{}] condition matched, handle matched case branch", self.node_name());
                    have_true_case = true;
                    cycle_tracker.end_phase(&phase_name);
                    self.handle_case_true(case, case_result.1, current_time).await?;
                    // Continue to process remaining cases as false (do not break)
                }
                // condition is false
                else {
                    // tracing::debug!("@[{}] condition not matched, send case false event", self.node_name());
                    self.handle_case_false(case).await;
                }
            } else {
                // A case has already matched, treat all subsequent cases as false
                // Skip condition evaluation and directly handle as not matched
                self.handle_case_false(case).await;
                cycle_tracker.end_phase(&phase_name);
            }
        }

        // If no case matched, handle the else branch
        if !have_true_case {
            // tracing::debug!("[{}] no case matched, handle else branch", self.node_name());
            let phase_name = format!("handle else branch");
            cycle_tracker.start_phase(&phase_name);
            self.handle_else_true().await;
            cycle_tracker.end_phase(&phase_name);
        } else {
            let phase_name = format!("handle else false");
            cycle_tracker.start_phase(&phase_name);
            self.handle_else_false().await;
            cycle_tracker.end_phase(&phase_name);
        }
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker)
            .await
            .unwrap();
        Ok(())
    }

    // 处理匹配的case
    async fn handle_case_true(
        &self,
        case: &Case,
        condition_results: Vec<ConditionResult>,
        current_time: DateTimeUtc,
    ) -> Result<(), IfElseNodeError> {
        let strategy_id = self.strategy_id().clone();
        let from_node_id = self.node_id().clone();
        let from_node_name = self.node_name().clone();
        let play_index = self.play_index();

        let case_output_handle_id = case.output_handle_id.clone();

        // 创建条件匹配事件
        let payload = CaseTruePayload::new(play_index, case.case_id);
        let condition_match_event: IfElseNodeEvent =
            CaseTrueEvent::new(from_node_id.clone(), from_node_name.clone(), case_output_handle_id.clone(), payload).into();

        // 创建并发送日志事件
        let condition_result_json = serde_json::to_value(condition_results).context(EvaluateResultSerializationFailedSnafu {
            node_name: self.node_name().clone(),
        })?;
        let message = ConditionMatchedMsg::new(from_node_name.clone(), case.case_id);
        let log_event: CommonEvent = StrategyRunningLogEvent::info_with_time(
            strategy_id,
            from_node_id.clone(),
            from_node_name.clone(),
            StrategyRunningLogSource::Node,
            StrategyRunningLogType::ConditionMatch,
            message.to_string(),
            condition_result_json,
            current_time,
        )
        .into();
        // let _ = strategy_output_handle.send(log_event.into());
        self.strategy_bound_handle_send(log_event.into()).unwrap();

        // 根据节点类型处理事件发送
        if self.is_leaf_node() {
            // 叶子节点：发送执行结束事件
            self.send_execute_over_event(self.play_index() as u64, Some(case.case_id)).unwrap();
        } else {
            // 非叶子节点：将事件传递给下游节点
            self.output_handle_send(&case_output_handle_id, condition_match_event.into())
                .unwrap();
            // let _ = case_output_handle.send(condition_match_event.into());
        }

        Ok(())
    }

    async fn handle_case_false(&self, case: &Case) {
        if self.is_leaf_node() {
            self.send_execute_over_event(self.play_index() as u64, Some(case.case_id)).unwrap();
            return;
        }

        let case_output_handle_id = case.output_handle_id.clone();
        let payload = CaseFalsePayload::new(self.play_index(), case.case_id);
        let case_false_event: IfElseNodeEvent = CaseFalseEvent::new(
            self.node_id().clone(),
            self.node_name().clone(),
            case_output_handle_id.clone(),
            payload,
        )
        .into();
        // tracing::debug!("@[{}] send case false event for case {}", self.node_name(), case.case_id);
        self.output_handle_send(&case_output_handle_id, case_false_event.into()).unwrap();
    }

    // 处理else分支
    async fn handle_else_true(&self) {
        // if self.is_leaf_node() {
        //     // 叶子节点：发送执行结束事件
        //     self.send_execute_over_event(self.play_index() as u64, None).unwrap();
        //     return;
        // }

        let else_output_handle = self.default_output_handle().unwrap();
        tracing::debug!(
            "handle_else_branch, else_output_handle: {:?}",
            else_output_handle.output_handle_id()
        );
        let payload = ElseTruePayload::new(self.play_index());
        let else_event: IfElseNodeEvent = ElseTrueEvent::new(
            self.node_id().clone(),
            self.node_name().clone(),
            else_output_handle.output_handle_id().clone(),
            payload,
        )
        .into();
        let _ = else_output_handle.send(else_event.into());
    }

    // 处理else分支
    async fn handle_else_false(&self) {
        // if self.is_leaf_node() {
        //     // 叶子节点：发送执行结束事件
        //     self.send_execute_over_event(self.play_index() as u64, None).unwrap();
        //     return;
        // }

        let else_output_handle = self.default_output_handle().unwrap();
        tracing::debug!("handle_else_false, else_output_handle: {:?}", else_output_handle.output_handle_id());
        let payload = ElseFalsePayload::new(self.play_index());
        let else_event: IfElseNodeEvent = ElseFalseEvent::new(
            self.node_id().clone(),
            self.node_name().clone(),
            else_output_handle.output_handle_id().clone(),
            payload,
        )
        .into();
        let _ = else_output_handle.send(else_event.into());
    }

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
        let left_value = parse_condition_left_value(&condition.left, received_value);

        // 获取右值
        let right_value = parse_condition_right_value(&condition.right, received_value);

        // 获取符号
        let comparison_symbol = &condition.comparison_symbol;

        let compare_result = compare(&left_value, &right_value, comparison_symbol);
        if compare_result {
            // tracing::debug!(
            //     "当前play_index: {}, 左变量名：{:?}, 左值={:?}, 比较符号:{:?}, 右变量名：{:?}, 右值={:?}, 结果={}",
            //     self.play_index(),
            //     condition.left.var_name,
            //     left_value,
            //     comparison_symbol.to_string(),
            //     condition.right,
            //     right_value,
            //     compare_result
            // );
        } else {
            tracing::warn!(
                "条件评估失败: 左值={:?}, 右值={:?}, 存在空值, 当前play_index: {}",
                left_value,
                right_value,
                self.play_index()
            );
        }

        ConditionResult::new(condition, left_value, right_value, compare_result)
    }

    // 评估and条件组
    async fn evaluate_and_conditions(&self, conditions: &Vec<Condition>) -> (bool, Vec<ConditionResult>) {
        // tracing::debug!("{}: 开始评估and条件组: {:#?}", self.node_id(), conditions);
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

        let payload = GetCurrentTimeCmdPayload;
        let cmd = GetCurrentTimeCommand::new(self.node_id().clone(), tx, payload);
        self.send_strategy_command(cmd.into()).await.unwrap();

        let response = rx.await.unwrap();
        match response {
            StrategyResponse::Success { payload, .. } => {
                return Ok(payload.current_time.clone());
            }
            StrategyResponse::Fail { error, .. } => {
                return Err(error.to_string());
            }
        }
    }
}
