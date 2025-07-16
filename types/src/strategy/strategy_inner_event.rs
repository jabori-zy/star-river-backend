use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{Sender, Receiver};





pub type StrategyInnerEventPublisher = Sender<StrategyInnerEvent>;
pub type StrategyInnerEventReceiver = Receiver<StrategyInnerEvent>;



#[derive(Debug, Clone, Serialize, Deserialize)]
// 策略内部事件
pub enum StrategyInnerEvent {
    PlayIndexUpdate(PlayIndexUpdateEvent), // 播放索引更新事件
}




#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayIndexUpdateEvent {
    pub total_signal_count: i32, // 总信号数量
    pub play_index: i32, // 播放索引
    pub timestamp: i64, // 时间戳
}




