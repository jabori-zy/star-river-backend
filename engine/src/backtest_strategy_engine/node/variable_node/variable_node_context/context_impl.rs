use super::{
    BacktestBaseNodeContext, BacktestNodeCommand, BacktestNodeContextTrait, BacktestNodeEvent, Command, Event, NodeOutputHandle,
    NodeResetResponse, VariableNodeContext, IfElseNodeEvent, TriggerConfig, ConditionTrigger
};
use async_trait::async_trait;
use event_center::event::node_event::NodeEventTrait;


use std::any::Any;

#[async_trait]
impl BacktestNodeContextTrait for VariableNodeContext {
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

    fn get_default_output_handle(&self) -> &NodeOutputHandle {
        let node_id = self.base_context.node_id.clone();
        self.base_context
            .output_handles
            .get(&format!("{}_default_output", node_id))
            .unwrap()
    }

    async fn handle_engine_event(&mut self, event: Event) {
        tracing::info!("{}: 处理事件: {:?}", self.get_node_id(), event);
        // match event {
        //     Event::Response(response_event) => {
        //         self.handle_response_event(response_event).await;
        //     }
        //     _ => {}
        // }
    }

    async fn handle_node_event(&mut self, node_event: BacktestNodeEvent) {
        
        match node_event {
            BacktestNodeEvent::IfElseNode(IfElseNodeEvent::ConditionMatch(match_event)) => {
                tracing::debug!("{}: 条件匹配，获取变量", self.get_node_name());
                // 过滤出condition trigger caseid相同的 的变量配置
                let condition_trigger_configs = self
                    .backtest_config
                    .variable_configs
                    .iter()
                    .filter(|config| matches!(config.trigger_config(), TriggerConfig::Condition(_)))
                    .filter(|config| match config.trigger_config() {
                        TriggerConfig::Condition(condition_trigger) => {
                            match condition_trigger {
                                ConditionTrigger::Case(case_trigger) => {
                                    if let Some(case_id) = match_event.case_id {
                                        case_trigger.case_id == case_id && &case_trigger.from_node_id == match_event.from_node_id()
                                    } else {
                                        false
                                    }
                                    
                                }
                                ConditionTrigger::Else(else_trigger) => {
                                    if match_event.case_id.is_none() {
                                        &else_trigger.from_node_id == match_event.from_node_id()
                                    } else {
                                        false
                                    }
                                }
                            }
                        }
                        _ => false,
                    })
                    .cloned()
                    .collect::<Vec<_>>();
                self.handle_condition_trigger_variable(&condition_trigger_configs).await;

            }
            _ => {}
        //         tracing::debug!("[{}] 条件匹配，获取变量", self.get_node_name());
        //         // 判断当前节点的模式
        //         // 如果是条件触发模式，则获取变量
        //         if self
        //             .backtest_config
        //             .variable_configs
        //             .iter()
        //             .any(|v| v.get_variable_type == GetVariableType::Condition)
        //         {
        //             tracing::info!("{}: 条件触发模式，获取变量", self.get_node_name());
        //             self.get_variable().await;
        //         }
        //     }
        //     BacktestNodeEvent::Common(CommonEvent::Trigger(_)) => {
        //         tracing::debug!("{}: 接受到trigger事件，不获取变量", self.get_node_name());
        //         // 叶子节点不发送trigger事件, 发送执行结束事件
        //         if self.is_leaf_node() {
        //             self.send_execute_over_event().await;
        //             return;
        //         }
        //         // 每个条件输出handle都发送trigger事件
        //         for config in self.backtest_config.variable_configs.iter() {
        //             let output_handle = self.get_output_handle(&config.output_handle_id);
        //             if output_handle.connect_count > 0 {
        //                 self.send_trigger_event(&config.output_handle_id).await;
        //             }
        //         }
        //         // 默认输出handle发送trigger事件
        //         let default_output_handle = self.get_default_output_handle();
        //         if default_output_handle.connect_count > 0 {
        //             self.send_trigger_event(&default_output_handle.output_handle_id).await;
        //         }
        //     }

        //     _ => {}
        }
    }

    async fn handle_node_command(&mut self, node_command: BacktestNodeCommand) {
        match node_command {
            BacktestNodeCommand::NodeReset(cmd) => {
                if self.get_node_id() == &cmd.node_id() {
                    let response = NodeResetResponse::success(self.get_node_id().clone(), None);
                    cmd.respond(response);
                }
            }
            _ => {}
        }
    }
}
