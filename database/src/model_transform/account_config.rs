// use types::account::AccountConfig;
use types::account::mt5_account::Mt5AccountConfig;
use types::account::BinanceAccountConfig;
use types::account::ExchangeAccountConfig;
use crate::entities::account_config::Model as AccountConfigModel;
use serde_json;
use types::market::Exchange;
use std::str::FromStr;
use crate::entities::mt5_account_config::Model as Mt5AccountConfigModel;
use crate::entities::mt5_account_info::Model as Mt5AccountInfoModel;
use types::account::mt5_account::Mt5AccountInfo;

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
            login: config.login,
            password: config.password,
            server: config.server,
            terminal_path: config.terminal_path,
            sort_index: config.sort_index,
            created_time: config.created_time,
            updated_time: config.updated_time,
        }
    }
}


impl From<Mt5AccountInfoModel> for Mt5AccountInfo {
    fn from(info: Mt5AccountInfoModel) -> Self {
        Mt5AccountInfo {
            id: info.id,
            account_id: info.account_id,
            login: info.login,
            trade_mode: info.trade_mode,
            leverage: info.leverage,
            limit_orders: info.limit_orders,
            margin_stopout_mode: info.margin_stopout_mode,
            trade_allowed: info.trade_allowed,
            dlls_allowed: info.dlls_allowed,
            terminal_connected: info.terminal_connected,
            trade_expert: info.trade_expert,
            margin_mode: info.margin_mode,
            currency_digits: info.currency_digits,
            fifo_close: info.fifo_close,
            balance: info.balance,
            credit: info.credit,
            profit: info.profit,
            equity: info.equity,
            margin: info.margin,
            margin_free: info.margin_free,
            margin_level: info.margin_level,
            margin_so_call: info.margin_so_call,
            margin_so_so: info.margin_so_so,
            margin_initial: info.margin_initial,
            margin_maintenance: info.margin_maintenance,
            assets: info.assets,
            liabilities: info.liabilities,
            commission_blocked: info.commission_blocked,
            name: info.name,
            server: info.server,
            currency: info.currency,
            company: info.company,
            created_time: info.created_time,
            updated_time: info.updated_time,
        }
    }
}




