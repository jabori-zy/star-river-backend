use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Receiver, Sender};

pub type StrategyInnerEventPublisher = Sender<StrategyInnerEvent>;
pub type StrategyInnerEventReceiver = Receiver<StrategyInnerEvent>;

#[derive(Debug, Clone, Serialize, Deserialize)]
// 策略内部事件
pub enum StrategyInnerEvent {
    // PlayIndexUpdate(PlayIndexUpdateEvent), // 播放索引更新事件
    NodeReset, // 节点重置事件
}
