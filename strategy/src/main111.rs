pub mod example;
pub mod node;
pub mod strategy;


use std::collections::HashMap;
use petgraph::Graph;
use example::*;


#[tokio::main]
async fn main() -> Result<(), String> {
    let mut graph = ExecutionGraph::new();

    // 添加节点
    let data_source_node = graph.add_node("data_source".to_string(), NodeType::DataSource);
    let sma12_node = graph.add_node("sma12".to_string(), NodeType::Indicator("SMA12".to_string()));
    let sma24_node = graph.add_node("sma24".to_string(), NodeType::Indicator("SMA24".to_string()));
    let condition_node = graph.add_node("condition".to_string(), NodeType::Condition);

    // 添加边
    graph.add_edge("data_source", "sma12");
    graph.add_edge("data_source", "sma24");
    graph.add_edge("sma12", "condition");
    graph.add_edge("sma24", "condition");

    let data_source_senders = graph.get_senders(data_source_node);
    println!("data_source_senders: {:?}", data_source_senders);
    let sma12_receivers = graph.get_receivers(sma12_node);
    println!("sma12_receivers: {:?}", sma12_receivers);
    let sma24_receivers = graph.get_receivers(sma24_node);
    println!("sma24_receivers: {:?}", sma24_receivers);
    let condition_receivers = graph.get_receivers(condition_node);
    println!("condition_receivers: {:?}", condition_receivers);


    // 运行策略
    graph.run().await;
    tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
    Ok(())


}
