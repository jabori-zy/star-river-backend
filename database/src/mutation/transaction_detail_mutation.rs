use types::transaction_detail::ExchangeTransactionDetail as TypeExchangeTransactionDetail;
use types::transaction_detail::TransactionDetail as TypeTransactionDetail;
use sea_orm::*;
use crate::entities::transaction_detail;



pub struct TransactionDetailMutation;


impl TransactionDetailMutation {
    
    pub async fn insert_transaction_detail(
        db: &DbConn,
        strategy_id: i64,
        node_id: String,
        transaction_detail: Box<dyn TypeExchangeTransactionDetail>
    ) -> Result<TypeTransactionDetail, DbErr> {
        let transaction_detail_model = transaction_detail::ActiveModel {
            id: NotSet,
            strategy_id: Set(strategy_id),
            node_id: Set(node_id),
            symbol: Set(transaction_detail.get_symbol()),
            exchange: Set(transaction_detail.get_exchange().to_string()),
            exchange_order_id: Set(transaction_detail.get_exchange_order_id()),
            exchange_position_id: Set(transaction_detail.get_exchange_position_id()),
            exchange_transaction_id: Set(transaction_detail.get_exchange_transaction_id()),
            transaction_type: Set(transaction_detail.get_transaction_type().to_string()),
            transaction_side: Set(transaction_detail.get_transaction_side().to_string()),
            quantity: Set(transaction_detail.get_quantity()),
            price: Set(transaction_detail.get_price()),
            created_time: Set(transaction_detail.get_create_time()),
            ..Default::default()
        }.insert(db).await.unwrap();

        Ok(transaction_detail_model.into())
    }
}
