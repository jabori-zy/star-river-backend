use exchange_core::ExchangeMetadata;
use star_river_core::custom_type::AccountId;

// ============================================================================
// Mt5Metadata Structure
// ============================================================================

/// MT5 exchange metadata
///
/// Stores MT5-specific configuration and runtime information
#[derive(Debug)]
pub struct BinanceMetadata {
    account_id: AccountId,
    account_name: String,
}

impl BinanceMetadata {
    pub fn new(account_id: AccountId, account_name: String) -> Self {
        Self { account_id, account_name }
    }

    pub fn account_id(&self) -> AccountId {
        self.account_id
    }

    pub fn account_name(&self) -> &String {
        &self.account_name
    }
}

impl ExchangeMetadata for BinanceMetadata {}
