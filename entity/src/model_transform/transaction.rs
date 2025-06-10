use crate::transaction::Model as TransactionModel;
use types::transaction::Transaction;
use types::market::Exchange;
use types::transaction::TransactionType;
use types::transaction::TransactionSide;
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
