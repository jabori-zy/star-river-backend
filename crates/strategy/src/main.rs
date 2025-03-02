pub mod node;
pub mod strategy;
pub mod data_source_node;
pub mod indicator_node;
pub mod condition_node;

use condition_node::{Condition, ConditionNode, ConditionType};
use node::*;
use strategy::*;
use types::market::*;
use types::indicator::*;
use types::indicator_config::*;

#[tokio::main]
async fn main() {
    // let data_source_node = DataSourceNode::new(Exchange::Binance, "BTCUSDT".to_string(), KlineInterval::Minutes1);
    let sma_config_14 = SMAConfig {    
        period: 14,
    };
    let sma_config_20 = SMAConfig {    
        period: 20,
    };
    // let indicator_node_14 = IndicatorNode::new(Indicators::SimpleMovingAverage(sma_config_14));
    // let indicator_node_20 = IndicatorNode::new(Indicators::SimpleMovingAverage(sma_config_20));

    let mut strategy = Strategy::new("test".to_string());
    let data_source_node_id = strategy.add_data_source_node("BTCUSDT".to_string(), Exchange::Binance, "BTCUSDT".to_string(), KlineInterval::Minutes1);
    let indicator_node_14_id = strategy.add_indicator_node("SMA14".to_string(), Indicators::SimpleMovingAverage(sma_config_14));
    let indicator_node_20_id = strategy.add_indicator_node("SMA20".to_string(), Indicators::SimpleMovingAverage(sma_config_20));

    // 添加条件
    let condition1 = Condition::new(indicator_node_14_id, ">", indicator_node_20_id);

    let mut condition_node = ConditionNode::new("condition_node".to_string(), ConditionType::And);
    condition_node.add_condition(condition1);
    // condition_node.add_condition(condition2);

    let condition_node_id = strategy.add_condition_node(condition_node);

    strategy.add_edge(&data_source_node_id, &indicator_node_14_id);
    strategy.add_edge(&data_source_node_id, &indicator_node_20_id);

    strategy.add_edge(&indicator_node_14_id, &condition_node_id);
    strategy.add_edge(&indicator_node_20_id, &condition_node_id);

        
    strategy.run().await;

    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;

}
