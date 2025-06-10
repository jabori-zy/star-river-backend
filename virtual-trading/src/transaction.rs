use super::VirtualTradingSystem;
use types::custom_type::*;



impl VirtualTradingSystem {
    pub fn get_transaction_id(&self) -> TransactionId {
        self.transactions.len() as TransactionId + 1
    }
}