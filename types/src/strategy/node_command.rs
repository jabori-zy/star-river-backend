use tokio::sync::{oneshot,mpsc};
use super::node_response::NodeResponse;

pub type NodeResponder = oneshot::Sender<NodeResponse>;
pub type NodeCommandSender = mpsc::Sender<NodeCommand>;
pub type NodeCommandReceiver = mpsc::Receiver<NodeCommand>;


#[derive(Debug)]
pub enum NodeCommand {
    Strategy(StrategyCommand),
}



#[derive(Debug)]
pub enum StrategyCommand {
    GetStrategyCacheKeys(GetStrategyCacheKeysParams),
}

impl From<StrategyCommand> for NodeCommand {
    fn from(command: StrategyCommand) -> Self {
        NodeCommand::Strategy(command)
    }
}

#[derive(Debug)]
pub struct GetStrategyCacheKeysParams {
    pub node_id: String,
    pub timestamp: i64,
    pub responder: NodeResponder,

}





