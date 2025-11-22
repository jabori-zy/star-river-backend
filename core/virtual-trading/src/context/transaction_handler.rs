use star_river_core::custom_type::*;

use super::VirtualTradingSystemContext;
use crate::types::VirtualTransaction;

impl<E> VirtualTradingSystemContext<E>
where
    E: Clone + Send + Sync + 'static,
{
    pub fn get_transaction_id(&self) -> TransactionId {
        self.transactions.len() as TransactionId + 1
    }

    pub fn get_transactions(&self) -> Vec<VirtualTransaction> {
        self.transactions.clone()
    }
}
