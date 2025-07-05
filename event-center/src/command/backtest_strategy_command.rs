use tokio::sync::{oneshot,mpsc};
use crate::response::backtest_strategy_response::StrategyResponse;
use crate::command::CommandTrait;
use crate::Responder;
use types::custom_type::NodeId;

// 策略命令响应器
pub type StrategyResponder = oneshot::Sender<StrategyResponse>;
// 策略命令发送器
pub type StrategyCommandSender = mpsc::Sender<StrategyCommand>;
// 策略命令接收器
pub type StrategyCommandReceiver = mpsc::Receiver<StrategyCommand>;


#[derive(Debug)]
pub enum StrategyCommand {
    GetStartNodeConfig(GetStartNodeConfigParams),
}


impl StrategyCommand {
    pub fn node_id(&self) -> &str {
        match self {
            StrategyCommand::GetStartNodeConfig(params) => &params.node_id,
        }
    }

    pub fn responder(&self) -> &StrategyResponder {
        match self {
            StrategyCommand::GetStartNodeConfig(params) => &params.responder,
        }
    }
}




#[derive(Debug)]
pub struct GetStartNodeConfigParams {
    pub node_id: NodeId,
    pub timestamp: i64,
    pub responder: StrategyResponder,

}