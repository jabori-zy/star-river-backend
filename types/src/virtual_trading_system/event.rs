use serde::{Serialize, Deserialize};
use strum::Display;
use tokio::sync::broadcast;
use crate::order::virtual_order::VirtualOrder;
use crate::position::virtual_position::VirtualPosition;

// 虚拟交易系统事件发送器
pub type VirtualTradingSystemEventSender = broadcast::Sender<VirtualTradingSystemEvent>;
// 虚拟交易系统事件接收器
pub type VirtualTradingSystemEventReceiver = broadcast::Receiver<VirtualTradingSystemEvent>;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum VirtualTradingSystemEvent {
    FuturesOrderCreated(VirtualOrder), // 订单已创建
    FuturesOrderFilled(VirtualOrder), // 订单已成交
    FuturesOrderCanceled(VirtualOrder), // 订单已取消
    
    PositionCreated(VirtualPosition), // 仓位已创建
    PositionUpdated(VirtualPosition), // 仓位已更新(价格变化, 止盈止损变化, 未实现盈亏变化)
}


