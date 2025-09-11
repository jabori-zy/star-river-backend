use crate::transaction::Model as TransactionModel;
use star_river_core::market::Exchange;
use star_river_core::transaction::Transaction;
use star_river_core::transaction::TransactionSide;
use star_river_core::transaction::TransactionType;
use std::str::FromStr;

impl From<TransactionModel> for Transaction {
    fn from(transaction: TransactionModel) -> Self {
        Transaction {
            transaction_id: transaction.id,
            symbol: transaction.symbol,
            exchange: Exchange::from_str(&transaction.exchange).unwrap(),
            exchange_order_id: transaction.exchange_order_id as i64,
            exchange_position_id: transaction.exchange_position_id as i64,
            exchange_transaction_id: transaction.exchange_transaction_id as i64,
            transaction_type: TransactionType::from_str(&transaction.transaction_type).unwrap(),
            transaction_side: TransactionSide::from_str(&transaction.transaction_side).unwrap(),
            quantity: transaction.quantity,
            price: transaction.price,
            create_time: transaction.created_time,
            extra_info: transaction.extra_info,
        }
    }
}
