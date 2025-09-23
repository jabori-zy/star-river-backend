use super::BacktestStrategyFunction;
use crate::strategy_engine::node::node_types::NodeType;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;
use event_center::communication::strategy::NodeCommandSender;
use serde_json::Value;
use snafu::Report;
use star_river_core::error::engine_error::strategy_engine_error::node_error::backtest_strategy_node_error::*;
use star_river_core::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use star_river_core::utils::camel_to_snake;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;

impl BacktestStrategyFunction {
    pub async fn add_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: Value,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), BacktestStrategyNodeError> {
        // 获取节点类型
        let node_type_str = camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                Self::add_start_node(context, node_config, node_command_sender, strategy_inner_event_receiver).await?;
                Ok(())
            }
            // k线节点
            NodeType::KlineNode => {
                Self::add_kline_node(context, node_config, node_command_sender, strategy_inner_event_receiver).await?;
                Ok(())
            }

            // // 指标节点
            NodeType::IndicatorNode => {
                Self::add_indicator_node(context, node_config, node_command_sender, strategy_inner_event_receiver)
                    .await?;
                Ok(())
            }

            // // 条件分支节点
            NodeType::IfElseNode => {
                Self::add_if_else_node(context, node_config, node_command_sender, strategy_inner_event_receiver)
                    .await?;
                Ok(())
            }
            // // 订单节点
            NodeType::FuturesOrderNode => {
                Self::add_futures_order_node(context, node_config, node_command_sender, strategy_inner_event_receiver)
                    .await?;
                Ok(())
            }
            // // 持仓节点
            NodeType::PositionManagementNode => {
                Self::add_position_management_node(
                    context,
                    node_config,
                    node_command_sender,
                    strategy_inner_event_receiver,
                )
                .await?;
                Ok(())
            }

            // }
            // // 获取变量节点
            NodeType::VariableNode => {
                Self::add_variable_node(context, node_config, node_command_sender, strategy_inner_event_receiver)
                    .await?;
                Ok(())
            }
            _ => {
                let error = UnsupportedNodeTypeSnafu {
                    node_type: node_type_str,
                }
                .build();
                let report = Report::from_error(&error);
                tracing::error!("{}", report);
                Err(error)
            }
        }
    }
}
