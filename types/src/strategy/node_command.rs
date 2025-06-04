use tokio::sync::{oneshot,mpsc};
use super::node_response::NodeResponse;

pub type NodeResponder = oneshot::Sender<NodeResponse>;
pub type NodeCommandSender = mpsc::Sender<NodeCommand>;
pub type NodeCommandReceiver = mpsc::Receiver<NodeCommand>;


#[derive(Debug)]
pub enum NodeCommand {
    GetStrategyCacheKeys(GetStrategyCacheKeysParams),
    GetKlineIndex(GetKlineIndexParams),
}

#[derive(Debug)]
pub struct GetStrategyCacheKeysParams {
    pub node_id: String,
    pub timestamp: i64,
    pub responder: NodeResponder,

}

#[derive(Debug)]
pub struct GetKlineIndexParams {
    pub node_id: String,
    pub timestamp: i64,
    pub responder: NodeResponder,
}
