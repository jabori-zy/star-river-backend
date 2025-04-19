// use types::account::AccountConfig;
use types::account::mt5_account::Mt5AccountConfig;
use types::account::BinanceAccountConfig;
use types::account::ExchangeAccountConfig;
use crate::entities::account_config::Model as AccountConfigModel;
use serde_json;
use types::market::Exchange;
use std::str::FromStr;
use crate::entities::mt5_account_config::Model as Mt5AccountConfigModel;



// impl From<AccountConfigModel> for AccountConfig {
//     fn from(config: AccountConfigModel) -> Self {

//         let exchange_account_config = match Exchange::from_str(config.exchange.as_str()).unwrap() {
//             Exchange::Metatrader5 => {
//                 // 将serde_json::Value转换为MetaTrader5AccountConfig
//                 let account_config = serde_json::from_value::<Mt5AccountConfig>(config.account_config).unwrap();
//                 ExchangeAccountConfig::MetaTrader5(account_config)
//             }
//             Exchange::Binance => {
//                 let account_config = serde_json::from_value::<BinanceAccountConfig>(config.account_config).unwrap();
//                 ExchangeAccountConfig::Binance(account_config)
//             }
//             _ => {
//                 panic!("Unsupported exchange: {}", config.exchange);
//             }
//         };

//         AccountConfig {
//             id: config.id,
//             account_name: config.account_name,
//             exchange: Exchange::from_str(config.exchange.as_str()).unwrap(),
//             is_available: config.is_available,
//             account_config: exchange_account_config,
//             created_time: config.created_time,
//             updated_time: config.updated_time,
//         }
//     }
// }


impl From<Mt5AccountConfigModel> for Mt5AccountConfig {
    fn from(config: Mt5AccountConfigModel) -> Self {
        Mt5AccountConfig {
            id: config.id,
            account_name: config.account_name,
            exchange: config.exchange,
            is_available: config.is_available,
            login: config.account_id,
            password: config.password,
            server: config.server,
            terminal_path: config.terminal_path,
            sort_index: config.sort_index,
            created_time: config.created_time,
            updated_time: config.updated_time,
        }
    }
}


