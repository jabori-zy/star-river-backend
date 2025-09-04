// pub mod live_strategy;
pub mod backtest_strategy;


use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;
use types::custom_type::NodeId;
use event_center::command::backtest_strategy_command::StrategyCommandSender;
use event_center::command::backtest_strategy_command::StrategyCommand;



#[derive(Clone, Debug)]
pub struct StrategyCommandPublisher {
    channels: Arc<Mutex<HashMap<NodeId, StrategyCommandSender>>>,
}

impl StrategyCommandPublisher {
    pub fn new() -> Self {
        Self { channels: Arc::new(Mutex::new(HashMap::new())) }
    }

    // 添加节点命令发送器
    pub async fn add_sender(&self, node_id: NodeId, sender: StrategyCommandSender) {
        let mut channels = self.channels.lock().await;
        channels.insert(node_id, sender);
    }

    // 发送命令
    pub async fn send(&self, command: StrategyCommand) -> Result<(), String> {
        let node_id = command.node_id();
        let channels = self.channels.lock().await;
        let sender = channels.get(node_id)
            .ok_or(format!("Node id {} not found", node_id))?;
        sender.send(command).await.map_err(|e| 
            format!("Failed to send command: {}", e)
        )?;
        Ok(())
    }
    
}



    




