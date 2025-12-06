use serde::{Deserialize, Serialize};
use strum::Display;
use tokio::sync::broadcast;

use crate::types::{order::VirtualOrder, position::VirtualPosition, transaction::VirtualTransaction};

// Virtual trading system event sender
pub type VtsEventSender = broadcast::Sender<VtsEvent>;
// Virtual trading system event receiver
pub type VtsEventReceiver = broadcast::Receiver<VtsEvent>;

#[derive(Debug, Clone, Serialize, Deserialize, Display)]
#[serde(tag = "event")]
pub enum VtsEvent {
    // All data updated
    UpdateFinished,

    LimitOrderExecutedDirectly { limit_price: f64, order: VirtualOrder }, // Limit order executed directly (limit price, order)

    // Order events
    FuturesOrderCreated(VirtualOrder),  // Order created
    FuturesOrderFilled(VirtualOrder),   // Order filled
    FuturesOrderCanceled(VirtualOrder), // Order canceled

    // Take profit order events
    TakeProfitOrderCreated(VirtualOrder),  // Take profit order created
    TakeProfitOrderFilled(VirtualOrder),   // Take profit order filled
    TakeProfitOrderCanceled(VirtualOrder), // Take profit order canceled

    // Stop loss order events
    StopLossOrderCreated(VirtualOrder),  // Stop loss order created
    StopLossOrderFilled(VirtualOrder),   // Stop loss order filled
    StopLossOrderCanceled(VirtualOrder), // Stop loss order canceled

    // Position events
    PositionCreated(VirtualPosition), // Position created
    PositionUpdated(VirtualPosition), // Position updated (price change, tp/sl change, unrealized pnl change)
    PositionClosed(VirtualPosition),  // Position closed

    // Transaction events
    TransactionCreated(VirtualTransaction), // Transaction created
}
