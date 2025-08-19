use serde::{Serialize, Deserialize};
use strum::Display;
use tokio::sync::broadcast;
use crate::order::virtual_order::VirtualOrder;
use crate::position::virtual_position::VirtualPosition;
use crate::transaction::virtual_transaction::VirtualTransaction;

// 虚拟交易系统事件发送器
pub type VirtualTradingSystemEventSender = broadcast::Sender<VirtualTradingSystemEvent>;
// 虚拟交易系统事件接收器
pub type VirtualTradingSystemEventReceiver = broadcast::Receiver<VirtualTradingSystemEvent>;



#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum VirtualTradingSystemEvent {
    // 所有数据已更新
    UpdateFinished, 

    // 订单事件
    FuturesOrderCreated(VirtualOrder), // 订单已创建
    FuturesOrderFilled(VirtualOrder), // 订单已成交
    FuturesOrderCanceled(VirtualOrder), // 订单已取消

    // 止盈订单事件
    TakeProfitOrderCreated(VirtualOrder), // 止盈订单已创建
    TakeProfitOrderFilled(VirtualOrder), // 止盈订单已成交
    TakeProfitOrderCanceled(VirtualOrder), // 止盈订单已取消

    // 止损订单事件
    StopLossOrderCreated(VirtualOrder), // 止损订单已创建
    StopLossOrderFilled(VirtualOrder), // 止损订单已成交
    StopLossOrderCanceled(VirtualOrder), // 止损订单已取消

    // 仓位事件
    PositionCreated(VirtualPosition), // 仓位已创建
    PositionUpdated(VirtualPosition), // 仓位已更新(价格变化, 止盈止损变化, 未实现盈亏变化)

    // 交易明细事件
    TransactionCreated(VirtualTransaction), // 交易明细已创建
}


