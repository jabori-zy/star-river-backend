use super::BacktestStrategyFunction;
use serde_json::Value;
use std::str::FromStr;
use crate::strategy_engine::node::node_types::NodeType;
use std::sync::Arc;
use event_center::EventReceiver;
use types::strategy::node_command::NodeCommandSender;
use types::strategy::strategy_inner_event::StrategyInnerEventReceiver;
use tokio::sync::RwLock;
use types::custom_type::PlayIndex;
use crate::strategy_engine::strategy::backtest_strategy::backtest_strategy_context::BacktestStrategyContext;

impl BacktestStrategyFunction {
    pub async fn add_node(
        context: Arc<RwLock<BacktestStrategyContext>>,
        node_config: Value,
        market_event_receiver: EventReceiver,
        response_event_receiver: EventReceiver,
        node_command_sender: NodeCommandSender,
        strategy_inner_event_receiver: StrategyInnerEventReceiver,
    ) -> Result<(), String> {
        // 获取节点类型
        let node_type_str = utils::camel_to_snake(node_config["type"].as_str().unwrap_or_default());
        let node_type = NodeType::from_str(&node_type_str).unwrap();
        // 根据节点类型，添加节点
        match node_type {
            NodeType::StartNode => {
                Self::add_start_node(
                    context,
                    node_config, 
                    node_command_sender, 
                    strategy_inner_event_receiver,
                ).await.unwrap();
                Ok(())
            }
            // k线节点
            NodeType::KlineNode => {
                Self::add_kline_node(
                    context,
                    node_config, 
                    market_event_receiver, 
                    response_event_receiver, 
                    node_command_sender,
                    strategy_inner_event_receiver,
                ).await.unwrap();
                Ok(())
            }
                
            // // 指标节点
            NodeType::IndicatorNode => {
                Self::add_indicator_node(
                    context, 
                    node_config, 
                    response_event_receiver,
                    node_command_sender, 
                    strategy_inner_event_receiver
                ).await.unwrap();
                Ok(())
                
            }
            
            // // 条件分支节点
            NodeType::IfElseNode => {
                Self::add_if_else_node(
                    context,
                    node_config,
                    node_command_sender,
                    strategy_inner_event_receiver,
                ).await.unwrap();
                Ok(())
            }
            // // 订单节点
            NodeType::FuturesOrderNode => {
                Self::add_futures_order_node(
                    context,
                    node_config, 
                    response_event_receiver, 
                    node_command_sender, 
                    strategy_inner_event_receiver, 
                ).await.unwrap();
                Ok(())
            }
            // // 持仓节点
            NodeType::PositionManagementNode => {
                Self::add_position_management_node(
                    context,
                    node_config, 
                    response_event_receiver,
                    node_command_sender, 
                    strategy_inner_event_receiver,
                ).await.unwrap();
                Ok(())
                
            }
                
            // }
            // // 获取变量节点
            NodeType::VariableNode => {
                Self::add_variable_node(
                    context,
                    node_config, 
                    response_event_receiver, 
                    node_command_sender, 
                    strategy_inner_event_receiver,
                ).await.unwrap();
                Ok(())
            }
            _ => {
                tracing::error!("不支持的节点类型: {}", node_type);
                Err("不支持的节点类型".to_string())
            }
            
        }

    }



    
    
}
