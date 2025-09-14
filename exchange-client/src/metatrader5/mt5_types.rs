use chrono::TimeZone;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use star_river_core::market::KlineInterval;
use star_river_core::market::{Exchange, MT5Server};
use star_river_core::order::CreateOrderParams;
use star_river_core::order::OrderStatus;
use star_river_core::order::OriginalOrder;
use star_river_core::order::{FuturesOrderSide, OrderType};
use star_river_core::position::GetPositionNumberParams;
use star_river_core::position::OriginalPosition;
use star_river_core::position::PositionSide;
use star_river_core::transaction::OriginalTransaction;
use star_river_core::transaction::{TransactionSide, TransactionType};
use std::any::Any;
use strum::{Display, EnumString};
use star_river_core::system::DateTimeUtc;

#[derive(Clone, Display, Serialize, Deserialize, Debug, EnumString, Eq, PartialEq, Hash)]
pub enum Mt5KlineInterval {
    #[strum(serialize = "M1")]
    Minutes1,
    #[strum(serialize = "M2")]
    Minutes2,
    #[strum(serialize = "M3")]
    Minutes3,
    #[strum(serialize = "M4")]
    Minutes4,
    #[strum(serialize = "M5")]
    Minutes5,
    #[strum(serialize = "M6")]
    Minutes6,
    #[strum(serialize = "M10")]
    Minutes10,
    #[strum(serialize = "M12")]
    Minutes12,
    #[strum(serialize = "M15")]
    Minutes15,
    #[strum(serialize = "M20")]
    Minutes20,
    #[strum(serialize = "M30")]
    Minutes30,
    #[strum(serialize = "H1")]
    Hours1,
    #[strum(serialize = "H2")]
    Hours2,
    #[strum(serialize = "H3")]
    Hours3,
    #[strum(serialize = "H4")]
    Hours4,
    #[strum(serialize = "H6")]
    Hours6,
    #[strum(serialize = "H8")]
    Hours8,
    #[strum(serialize = "H12")]
    Hours12,
    #[strum(serialize = "D1")]
    Days1,
    #[strum(serialize = "W1")]
    Weeks1,
    #[strum(serialize = "MN1")]
    Months1,
}

// 将KlineInterval转换为BinanceKlineInterval
impl From<KlineInterval> for Mt5KlineInterval {
    fn from(interval: KlineInterval) -> Self {
        match interval {
            KlineInterval::Minutes1 => Mt5KlineInterval::Minutes1,
            KlineInterval::Minutes2 => Mt5KlineInterval::Minutes2,
            KlineInterval::Minutes3 => Mt5KlineInterval::Minutes3,
            KlineInterval::Minutes4 => Mt5KlineInterval::Minutes4,
            KlineInterval::Minutes5 => Mt5KlineInterval::Minutes5,
            KlineInterval::Minutes6 => Mt5KlineInterval::Minutes6,
            KlineInterval::Minutes10 => Mt5KlineInterval::Minutes10,
            KlineInterval::Minutes12 => Mt5KlineInterval::Minutes12,
            KlineInterval::Minutes15 => Mt5KlineInterval::Minutes15,
            KlineInterval::Minutes20 => Mt5KlineInterval::Minutes20,
            KlineInterval::Minutes30 => Mt5KlineInterval::Minutes30,
            KlineInterval::Hours1 => Mt5KlineInterval::Hours1,
            KlineInterval::Hours2 => Mt5KlineInterval::Hours2,
            KlineInterval::Hours3 => Mt5KlineInterval::Hours3,
            KlineInterval::Hours4 => Mt5KlineInterval::Hours4,
            KlineInterval::Hours6 => Mt5KlineInterval::Hours6,
            KlineInterval::Hours8 => Mt5KlineInterval::Hours8,
            KlineInterval::Hours12 => Mt5KlineInterval::Hours12,
            KlineInterval::Days1 => Mt5KlineInterval::Days1,
            KlineInterval::Weeks1 => Mt5KlineInterval::Weeks1,
            KlineInterval::Months1 => Mt5KlineInterval::Months1,
        }
    }
}

// 将BinanceKlineInterval转换为KlineInterval
impl From<Mt5KlineInterval> for KlineInterval {
    fn from(value: Mt5KlineInterval) -> Self {
        match value {
            Mt5KlineInterval::Minutes1 => KlineInterval::Minutes1,
            Mt5KlineInterval::Minutes2 => KlineInterval::Minutes2,
            Mt5KlineInterval::Minutes3 => KlineInterval::Minutes3,
            Mt5KlineInterval::Minutes4 => KlineInterval::Minutes4,
            Mt5KlineInterval::Minutes5 => KlineInterval::Minutes5,
            Mt5KlineInterval::Minutes6 => KlineInterval::Minutes6,
            Mt5KlineInterval::Minutes10 => KlineInterval::Minutes10,
            Mt5KlineInterval::Minutes12 => KlineInterval::Minutes12,
            Mt5KlineInterval::Minutes15 => KlineInterval::Minutes15,
            Mt5KlineInterval::Minutes20 => KlineInterval::Minutes20,
            Mt5KlineInterval::Minutes30 => KlineInterval::Minutes30,
            Mt5KlineInterval::Hours1 => KlineInterval::Hours1,
            Mt5KlineInterval::Hours2 => KlineInterval::Hours2,
            Mt5KlineInterval::Hours3 => KlineInterval::Hours3,
            Mt5KlineInterval::Hours4 => KlineInterval::Hours4,
            Mt5KlineInterval::Hours6 => KlineInterval::Hours6,
            Mt5KlineInterval::Hours8 => KlineInterval::Hours8,
            Mt5KlineInterval::Hours12 => KlineInterval::Hours12,
            Mt5KlineInterval::Days1 => KlineInterval::Days1,
            Mt5KlineInterval::Weeks1 => KlineInterval::Weeks1,
            Mt5KlineInterval::Months1 => KlineInterval::Months1,
        }
    }
}

impl Mt5KlineInterval {
    pub const ALL: &'static [Mt5KlineInterval] = &[
        Mt5KlineInterval::Minutes1,
        Mt5KlineInterval::Minutes2,
        Mt5KlineInterval::Minutes3,
        Mt5KlineInterval::Minutes4,
        Mt5KlineInterval::Minutes5,
        Mt5KlineInterval::Minutes6,
        Mt5KlineInterval::Minutes10,
        Mt5KlineInterval::Minutes12,
        Mt5KlineInterval::Minutes15,
        Mt5KlineInterval::Minutes20,
        Mt5KlineInterval::Minutes30,
        Mt5KlineInterval::Hours1,
        Mt5KlineInterval::Hours2,
        Mt5KlineInterval::Hours3,
        Mt5KlineInterval::Hours4,
        Mt5KlineInterval::Hours6,
        Mt5KlineInterval::Hours8,
        Mt5KlineInterval::Hours12,
        Mt5KlineInterval::Days1,
        Mt5KlineInterval::Weeks1,
        Mt5KlineInterval::Months1,
    ];

    pub fn to_list() -> &'static [Mt5KlineInterval] {
        Self::ALL
    }
}

#[derive(Debug, Serialize)]
pub struct Mt5CreateOrderParams {
    pub symbol: String,
    pub order_type: String,
    pub order_side: String,
    pub volume: f64,
    pub price: f64,
    pub tp: Option<f64>,
    pub sl: Option<f64>,
}

impl From<CreateOrderParams> for Mt5CreateOrderParams {
    fn from(value: CreateOrderParams) -> Self {
        Mt5CreateOrderParams {
            symbol: value.symbol,
            order_type: value.order_type.to_string(),
            order_side: value.order_side.to_string(),
            volume: value.quantity,
            price: value.price,
            tp: value.tp,
            sl: value.sl,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Mt5GetPositionNumberParams {
    pub exchange: Exchange,
    pub symbol: String,
    pub position_side: Option<PositionSide>,
}

impl From<GetPositionNumberParams> for Mt5GetPositionNumberParams {
    fn from(value: GetPositionNumberParams) -> Self {
        Mt5GetPositionNumberParams {
            exchange: value.exchange,
            symbol: value.symbol,
            position_side: value.position_side,
        }
    }
}

// mt5 订单状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "lowercase")]
pub enum Mt5OrderState {
    #[strum(serialize = "created")] // 已创建
    Created,
    #[strum(serialize = "placed")] // 已挂单
    Placed,
    #[strum(serialize = "filled")] // 已成交
    Filled,
    #[strum(serialize = "partial")] // 部分成交
    Partial,
    #[strum(serialize = "canceled")] // 已取消
    Canceled,
    #[strum(serialize = "expired")] // 已过期
    Expired,
    #[strum(serialize = "rejected")] // 已拒绝
    Rejected,
}

impl From<Mt5OrderState> for OrderStatus {
    fn from(value: Mt5OrderState) -> Self {
        match value {
            Mt5OrderState::Created => OrderStatus::Created,
            Mt5OrderState::Placed => OrderStatus::Placed,
            Mt5OrderState::Filled => OrderStatus::Filled,
            Mt5OrderState::Partial => OrderStatus::Partial,
            Mt5OrderState::Canceled => OrderStatus::Canceled,
            Mt5OrderState::Expired => OrderStatus::Expired,
            Mt5OrderState::Rejected => OrderStatus::Rejected,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "snake_case")]
pub enum Mt5OrderType {
    #[strum(serialize = "order_type_buy")]
    OrderTypeBuy,
    #[strum(serialize = "order_type_sell")]
    OrderTypeSell,
    #[strum(serialize = "order_type_buy_limit")]
    OrderTypeBuyLimit,
    #[strum(serialize = "order_type_sell_limit")]
    OrderTypeSellLimit,
    #[strum(serialize = "order_type_buy_stop")]
    OrderTypeBuyStop,
    #[strum(serialize = "order_type_sell_stop")]
    OrderTypeSellStop,
    #[strum(serialize = "order_type_buy_stop_limit")]
    OrderTypeBuyStopLimit,
    #[strum(serialize = "order_type_sell_stop_limit")]
    OrderTypeSellStopLimit,
    #[strum(serialize = "order_type_close_by")]
    OrderTypeCloseBy,
}

impl From<Mt5OrderType> for OrderType {
    fn from(value: Mt5OrderType) -> Self {
        match value {
            Mt5OrderType::OrderTypeBuy => OrderType::Market,
            Mt5OrderType::OrderTypeSell => OrderType::Market,
            Mt5OrderType::OrderTypeBuyLimit => OrderType::Limit,
            Mt5OrderType::OrderTypeSellLimit => OrderType::Limit,
            Mt5OrderType::OrderTypeBuyStop => OrderType::StopMarket,
            Mt5OrderType::OrderTypeSellStop => OrderType::StopMarket,
            Mt5OrderType::OrderTypeBuyStopLimit => OrderType::TakeProfitMarket,
            Mt5OrderType::OrderTypeSellStopLimit => OrderType::TakeProfitMarket,
            Mt5OrderType::OrderTypeCloseBy => {
                tracing::warn!("遇到不支持的订单类型 OrderTypeCloseBy, 将转换为默认市价单类型");
                OrderType::Market // 设置默认转换为市价单
            }
        }
    }
}

impl From<Mt5OrderType> for FuturesOrderSide {
    fn from(value: Mt5OrderType) -> Self {
        match value {
            Mt5OrderType::OrderTypeBuy => FuturesOrderSide::OpenLong,
            Mt5OrderType::OrderTypeSell => FuturesOrderSide::OpenShort,
            Mt5OrderType::OrderTypeBuyLimit => FuturesOrderSide::OpenLong,
            Mt5OrderType::OrderTypeSellLimit => FuturesOrderSide::OpenShort,
            Mt5OrderType::OrderTypeBuyStop => FuturesOrderSide::OpenLong,
            Mt5OrderType::OrderTypeSellStop => FuturesOrderSide::OpenShort,
            Mt5OrderType::OrderTypeBuyStopLimit => FuturesOrderSide::OpenLong,
            Mt5OrderType::OrderTypeSellStopLimit => FuturesOrderSide::OpenShort,
            Mt5OrderType::OrderTypeCloseBy => FuturesOrderSide::OpenLong,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Mt5Order {
    pub server: MT5Server,
    pub order_id: i64,
    pub position_id: i64,
    pub symbol: String,
    pub time_setup: i64,
    pub time_setup_msc: i64,
    pub time_done: i64,
    pub time_done_msc: i64,
    pub time_expiration: i64,
    pub order_type: Mt5OrderType,
    pub type_time: String,
    pub type_filling: String,
    pub state: Mt5OrderState,
    pub reason: String,
    pub volume_initial: f64,
    pub volume_current: f64,
    pub open_price: f64,
    pub sl: Option<f64>,
    pub tp: Option<f64>,
    pub comment: String,
}

impl OriginalOrder for Mt5Order {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn OriginalOrder> {
        Box::new(self.clone())
    }
    fn get_exchange_order_id(&self) -> i64 {
        self.order_id
    }
    fn get_exchange(&self) -> Exchange {
        Exchange::Metatrader5(self.server.clone())
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_order_side(&self) -> FuturesOrderSide {
        let order_side: FuturesOrderSide = self.order_type.clone().into();
        order_side.clone()
    }
    fn get_order_type(&self) -> OrderType {
        let order_type: OrderType = self.order_type.clone().into();
        order_type
    }
    fn get_order_status(&self) -> OrderStatus {
        let order_status: OrderStatus = self.state.clone().into();
        order_status
    }
    fn get_quantity(&self) -> f64 {
        self.volume_initial
    }
    fn get_open_price(&self) -> f64 {
        self.open_price
    }
    fn get_tp(&self) -> Option<f64> {
        self.tp
    }
    fn get_sl(&self) -> Option<f64> {
        self.sl
    }

    fn get_extra_info(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "server": self.server,
            "order_id": self.order_id,
            "position_id": self.position_id,
            "symbol": self.symbol,
            "time_setup": self.time_setup,
            "reason": self.reason,
        }))
    }
    fn get_created_time(&self) -> DateTimeUtc {
        // 把时间戳转换为日期时间
        let created_time = Utc
            .timestamp_millis_opt(self.time_setup_msc)
            .single()
            .expect("时间戳转换为日期时间失败");
        created_time
    }

    fn get_updated_time(&self) -> DateTimeUtc {
        let updated_time = Utc
            .timestamp_millis_opt(self.time_done_msc)
            .single()
            .expect("时间戳转换为日期时间失败");
        updated_time
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "snake_case")]
pub enum Mt5PositionSide {
    #[strum(serialize = "long")]
    Long,
    #[strum(serialize = "short")]
    Short,
}

impl From<Mt5PositionSide> for PositionSide {
    fn from(value: Mt5PositionSide) -> Self {
        match value {
            Mt5PositionSide::Long => PositionSide::Long,
            Mt5PositionSide::Short => PositionSide::Short,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Mt5Position {
    pub server: MT5Server,
    pub position_id: i64,
    pub time: i64,
    pub time_msc: i64,
    pub time_update: i64,
    pub time_update_msc: i64,
    #[serde(rename = "position_type")]
    pub position_side: Mt5PositionSide,
    pub magic: i64,
    pub identifier: i64,
    pub reason: String,
    pub volume: f64,
    pub open_price: f64,
    pub sl: f64,
    pub tp: f64,
    pub current_price: f64,
    pub swap: f64,
    pub profit: f64,
    pub symbol: String,
    pub comment: String,
    pub external_id: String,
}

impl OriginalPosition for Mt5Position {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn OriginalPosition> {
        Box::new(self.clone())
    }
    fn get_exchange_position_id(&self) -> i64 {
        self.position_id
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_position_side(&self) -> PositionSide {
        self.position_side.clone().into()
    }
    fn get_quantity(&self) -> f64 {
        self.volume
    }
    fn get_open_price(&self) -> f64 {
        self.open_price
    }
    fn get_tp(&self) -> Option<f64> {
        Some(self.tp)
    }
    fn get_sl(&self) -> Option<f64> {
        Some(self.sl)
    }
    fn get_exchange(&self) -> Exchange {
        Exchange::Metatrader5(self.server.clone())
    }
    fn get_unrealized_profit(&self) -> Option<f64> {
        Some(self.profit)
    }

    fn get_extra_info(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "magic": self.magic,
            "identifier": self.identifier,
            "reason": self.reason,
            "comment": self.comment,
            "external_id": self.external_id,
        }))
    }
    fn get_create_time(&self) -> DateTimeUtc {
        Utc.timestamp_millis_opt(self.time_msc)
            .single()
            .expect("时间戳转换为日期时间失败")
    }
    fn get_update_time(&self) -> DateTimeUtc {
        Utc.timestamp_millis_opt(self.time_update_msc)
            .single()
            .expect("时间戳转换为日期时间失败")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "lowercase")]
pub enum Mt5DealType {
    #[strum(serialize = "buy")]
    Buy,
    #[strum(serialize = "sell")]
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize, EnumString, Display)]
#[serde(rename_all = "lowercase")]
pub enum Mt5DealEntry {
    #[strum(serialize = "in")]
    In,
    #[strum(serialize = "out")]
    Out,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Mt5Deal {
    pub server: MT5Server,
    pub deal_id: i64,
    pub order_id: i64,
    pub position_id: i64,
    pub symbol: String,
    pub time: i64,
    pub time_msc: i64,
    pub deal_type: Mt5DealType,
    pub entry: Mt5DealEntry,
    pub magic: i64,
    pub deal_reason: String,
    pub volume: f64,
    pub price: f64,
    pub commission: f64,
    pub swap: f64,
    pub profit: f64,
    pub fee: f64,
    pub comment: String,
    pub external_id: String,
}

impl OriginalTransaction for Mt5Deal {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn OriginalTransaction> {
        Box::new(self.clone())
    }
    fn get_transaction_id(&self) -> i64 {
        self.deal_id
    }
    fn get_transaction_type(&self) -> TransactionType {
        match self.entry {
            Mt5DealEntry::In => TransactionType::Open,
            Mt5DealEntry::Out => TransactionType::Close,
        }
    }
    fn get_transaction_side(&self) -> TransactionSide {
        match self.deal_type {
            Mt5DealType::Buy => TransactionSide::OpenLong,
            Mt5DealType::Sell => TransactionSide::OpenShort,
        }
    }
    fn get_quantity(&self) -> f64 {
        self.volume
    }
    fn get_price(&self) -> f64 {
        self.price
    }
    fn get_create_time(&self) -> DateTimeUtc {
        Utc.timestamp_millis_opt(self.time_msc)
            .single()
            .expect("时间戳转换为日期时间失败")
    }
    fn get_symbol(&self) -> String {
        self.symbol.clone()
    }
    fn get_exchange(&self) -> Exchange {
        Exchange::Metatrader5(self.server.clone())
    }
    fn get_exchange_order_id(&self) -> i64 {
        self.order_id
    }
    fn get_exchange_position_id(&self) -> i64 {
        self.position_id
    }
    fn get_exchange_transaction_id(&self) -> i64 {
        self.deal_id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mt5Response<T> {
    pub success: bool,
    pub message: Option<String>,
    pub data: T,
    pub code: Option<i64>,
    pub details: Option<String>,
}
