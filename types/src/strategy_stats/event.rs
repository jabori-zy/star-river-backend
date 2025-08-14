use serde::{Serialize, Deserialize};
use strum::Display;
use tokio::sync::broadcast;




// 策略统计事件发送器
pub type StrategyStatsEventSender = broadcast::Sender<StrategyStatsEvent>;
// 策略统计事件接收器
pub type StrategyStatsEventReceiver = broadcast::Receiver<StrategyStatsEvent>;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum StrategyStatsEvent {
    PlayIndexUpdated(i32), // 播放索引已更新
}
