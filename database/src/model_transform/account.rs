use types::account::AccountConfig;
use crate::entities::account_config::Model as AccountConfigModel;




impl From<AccountConfigModel> for AccountConfig {
    fn from(account_config: AccountConfigModel) -> Self {
        AccountConfig {
            id: account_config.id,
            account_name: account_config.account_name,
            exchange: account_config.exchange,
            is_available: account_config.is_available,
            account_config: account_config.account_config,
            created_time: account_config.created_time,
            updated_time: account_config.updated_time,
        }
    }
}
