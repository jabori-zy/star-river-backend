use ::entity::transaction;
use sea_orm::*;
use types::transaction::OriginalTransaction;
use types::transaction::Transaction;

pub struct TransactionMutation;

impl TransactionMutation {
    pub async fn insert_transaction(
        db: &DbConn,
        strategy_id: i64,
        node_id: String,
        transaction: Box<dyn OriginalTransaction>,
    ) -> Result<Transaction, DbErr> {
        let transaction_model = transaction::ActiveModel {
            id: NotSet,
            strategy_id: Set(strategy_id),
            node_id: Set(node_id),
            symbol: Set(transaction.get_symbol()),
            exchange: Set(transaction.get_exchange().to_string()),
            exchange_order_id: Set(transaction.get_exchange_order_id()),
            exchange_position_id: Set(transaction.get_exchange_position_id()),
            exchange_transaction_id: Set(transaction.get_exchange_transaction_id()),
            transaction_type: Set(transaction.get_transaction_type().to_string()),
            transaction_side: Set(transaction.get_transaction_side().to_string()),
            quantity: Set(transaction.get_quantity()),
            price: Set(transaction.get_price()),
            created_time: Set(transaction.get_create_time()),
            ..Default::default()
        }
        .insert(db)
        .await
        .unwrap();

        Ok(transaction_model.into())
    }
}
