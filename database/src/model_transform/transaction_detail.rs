
use crate::entities::transaction_detail::Model as TransactionDetailModel;
use types::transaction_detail::TransactionDetail;
use types::market::Exchange;
use types::transaction_detail::TransactionType;
use types::transaction_detail::TransactionSide;
use std::str::FromStr;


impl From<TransactionDetailModel> for TransactionDetail {
    fn from(transaction_detail: TransactionDetailModel) -> Self {
        TransactionDetail {
            transaction_id: transaction_detail.id as i64,
            symbol: transaction_detail.symbol,
            exchange: Exchange::from_str(&transaction_detail.exchange).unwrap(),
            exchange_order_id: transaction_detail.exchange_order_id as i64,
            exchange_position_id: transaction_detail.exchange_position_id as i64,
            exchange_transaction_id: transaction_detail.exchange_transaction_id as i64,
            transaction_type: TransactionType::from_str(&transaction_detail.transaction_type).unwrap(),
            transaction_side: TransactionSide::from_str(&transaction_detail.transaction_side).unwrap(),
            quantity: transaction_detail.quantity,
            price: transaction_detail.price,
            create_time: transaction_detail.created_time,
        }
    }
}
