// External crate imports
use snafu::ResultExt;
use tokio::sync::oneshot;

// Current crate imports - star_river_core
use star_river_core::custom_type::NodeId;
use star_river_core::system::DateTimeUtc;

// Current crate imports - star_river_event
use star_river_event::backtest_strategy::node_event::IfElseNodeEvent;
use star_river_event::backtest_strategy::node_event::if_else_node_event::{
    ConditionMatchEvent,
    ConditionMatchPayload,
};

// Current crate imports - strategy_core
use strategy_core::benchmark::node_benchmark::CycleTracker;
use strategy_core::communication::strategy::StrategyResponse;
use strategy_core::event::log_event::{
    StrategyRunningLogEvent,
    StrategyRunningLogSource,
    StrategyRunningLogType,
};
use strategy_core::event::node_common_event::{
    CommonEvent,
    TriggerEvent,
    TriggerPayload,
};
use strategy_core::node::context_trait::{
    NodeBenchmarkExt,
    NodeCommunicationExt,
    NodeHandleExt,
    NodeIdentityExt,
    NodeRelationExt,
};
use strategy_core::node_infra::if_else_node::{
    Case,
    Condition,
    ConditionResult,
    FormulaRight,
    LogicalSymbol,
};

// Local module imports
use crate::node::node_error::IfElseNodeError;
use crate::node::node_error::if_else_node_error::EvaluateResultSerializationFailedSnafu;
use crate::node::node_message::if_else_node_log_message::ConditionMatchedMsg;
use crate::node_catalog::if_else_node::utils::{
    compare,
    parse_condition_left_value,
    parse_condition_right_value,
};
use crate::strategy::strategy_command::{
    GetCurrentTimeCommand,
    GetCurrentTimeCmdPayload,
};

// Relative imports
use super::{ConfigId, IfElseNodeContext, IfElseNodeBacktestConfig};

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

    // 开始评估各个分支
    pub async fn evaluate(&mut self) -> Result<(), IfElseNodeError> {
        let mut cycle_tracker = CycleTracker::new(self.play_index() as u32);


        let mut case_matched = false; // 是否匹配到case
        let current_time = self.get_current_time().await.unwrap();

        // 遍历case进行条件评估
        for (index, case) in self.node_config.cases.iter().enumerate() {
            let phase_name = format!("evaluate case {}", case.case_id);
            cycle_tracker.start_phase(&phase_name);
            let case_result = self.evaluate_case(case).await;

            // 如果条件匹配，处理匹配的case
            if case_result.0 {
                tracing::debug!("[{}] 条件匹配，处理匹配的case 分支", self.node_name());
                self.handle_matched_case(case, case_result.1, current_time).await?;
            }
            // 如果条件不匹配，并且是最后一个case, 则发送trigger事件
            else {
                if index == self.node_config.cases.len() - 1 {
                    tracing::debug!("[{}] 条件不匹配，发送trigger事件", self.node_name());
                    self.handle_not_matched_case(case).await;
                }
            }
            case_matched = true;
            cycle_tracker.end_phase(&phase_name);
            break; // 找到匹配的case后立即退出
        }

        // 如果没有case匹配，处理else分支
        if !case_matched {
            tracing::debug!("[{}] 条件不匹配，处理else分支", self.node_name());
            let phase_name = format!("handle else branch");
            cycle_tracker.start_phase(&phase_name);
            self.handle_else_branch().await;
            cycle_tracker.end_phase(&phase_name);
        }
        let completed_tracker = cycle_tracker.end();
        self.mount_node_cycle_tracker(self.node_id().clone(), completed_tracker).await;
        Ok(())
    }

    // 处理匹配的case
    async fn handle_matched_case(
        &self,
        case: &Case,
        condition_results: Vec<ConditionResult>,
        current_time: DateTimeUtc,
    ) -> Result<(), IfElseNodeError> {
        let strategy_id = self.strategy_id().clone();
        let from_node_id = self.node_id().clone();
        let from_node_name = self.node_name().clone();
        let play_index = self.play_index();

        let case_output_handle_id = format!("{}_output_{}", self.node_id(), case.case_id);
        let case_output_handle = self.output_handle(&case_output_handle_id).unwrap();
        let strategy_output_handle = self.strategy_bound_handle();

        // 创建条件匹配事件
        let payload = ConditionMatchPayload::new(play_index, Some(case.case_id));
        let condition_match_event: IfElseNodeEvent =
            ConditionMatchEvent::new(from_node_id.clone(), from_node_name.clone(), case_output_handle_id, payload).into();

        // 创建并发送日志事件
        let condition_result_json = serde_json::to_value(condition_results).context(EvaluateResultSerializationFailedSnafu {node_name: self.node_name().clone()})?;
        let message = ConditionMatchedMsg::new(from_node_name.clone(), case.case_id);
        let log_event: CommonEvent = StrategyRunningLogEvent::success(
            strategy_id,
            from_node_id.clone(),
            from_node_name.clone(),
            StrategyRunningLogSource::Node,
            StrategyRunningLogType::ConditionMatch,
            message.to_string(),
            condition_result_json,
            current_time,
        ).into();
        let _ = strategy_output_handle.send(log_event.into());

        // 根据节点类型处理事件发送
        if self.is_leaf_node() {
            // 叶子节点：发送执行结束事件
            self.send_execute_over_event();
        } else {
            // 非叶子节点：将事件传递给下游节点
            let _ = case_output_handle.send(condition_match_event.into());
        }

        Ok(())
    }

    async fn handle_not_matched_case(&self, case: &Case) {
        if self.is_leaf_node() {
            self.send_execute_over_event();
            return;
        }

        let case_output_handle_id = format!("{}_output_{}", self.node_id(), case.case_id);
        let case_output_handle = self.output_handle(&case_output_handle_id).unwrap();
        let payload = TriggerPayload::new(self.play_index() as u64);

        let condition_not_match_event: CommonEvent = TriggerEvent::new(
            self.node_id().clone(),
            self.node_name().clone(),
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
            self.send_execute_over_event();
        } else {
            // 非叶子节点：发送事件到else输出句柄
            let else_output_handle = self.default_output_handle().unwrap();
            let payload = ConditionMatchPayload::new(self.play_index(), None);
            let else_event: IfElseNodeEvent = ConditionMatchEvent::new(
                self.node_id().clone(),
                self.node_name().clone(),
                else_output_handle.output_handle_id().clone(),
                payload,
            )
            .into();
            let _ = else_output_handle.send(else_event.into());
        }
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
            tracing::debug!(
                "当前play_index: {}, 左变量名：{:?}, 左值={:?}, 比较符号:{:?}, 右变量名：{:?}, 右值={:?}, 结果={}",
                self.play_index(),
                condition.left.var_name,
                left_value,
                comparison_symbol.to_string(),
                condition.right,
                right_value,
                compare_result
            );
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