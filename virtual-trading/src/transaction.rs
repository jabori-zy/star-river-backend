use super::VirtualTradingSystem;
use types::custom_type::*;
use types::transaction::virtual_transaction::VirtualTransaction;

impl VirtualTradingSystem {
    pub fn get_transaction_id(&self) -> TransactionId {
        self.transactions.len() as TransactionId + 1
    }

    pub fn get_transactions(&self) -> Vec<VirtualTransaction> {
        self.transactions.clone()
    }
}
