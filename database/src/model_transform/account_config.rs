use types::account::AccountConfig;
use crate::entities::account_config::Model as AccountConfigModel;
use types::market::Exchange;
use std::str::FromStr;

impl From<AccountConfigModel> for AccountConfig {
    fn from(config: AccountConfigModel) -> Self {
        let exchange = match config.exchange.as_str() {
            "metatrader5" => Exchange::Metatrader5(config.account_config["server"].as_str().unwrap_or("").to_string()),
            _ => Exchange::from_str(config.exchange.as_str()).unwrap(),
        };
        AccountConfig {
            id: config.id,
            account_name: config.account_name,
            exchange: exchange,
            is_available: config.is_available,
            is_deleted: config.is_delete,
            sort_index: config.sort_index,
            config: config.account_config,
            create_time: config.create_time,
            update_time: config.update_time,
        }
    }
}


// impl From<Mt5AccountConfigModel> for Mt5AccountConfig {
//     fn from(config: Mt5AccountConfigModel) -> Self {
//         Mt5AccountConfig {
//             id: config.id,
//             account_name: config.account_name,
//             exchange: config.exchange,
//             is_available: config.is_available,
//             login: config.login,
//             password: config.password,
//             server: config.server,
//             terminal_path: config.terminal_path,
//             sort_index: config.sort_index,
//             created_time: config.created_time,
//             updated_time: config.updated_time,
//         }
//     }
// }


// impl From<Mt5AccountInfoModel> for Mt5AccountInfo {
//     fn from(info: Mt5AccountInfoModel) -> Self {
//         Mt5AccountInfo {
//             id: info.id,
//             account_id: info.account_id,
//             login: info.login,
//             trade_mode: info.trade_mode,
//             leverage: info.leverage,
//             limit_orders: info.limit_orders,
//             margin_stopout_mode: info.margin_stopout_mode,
//             trade_allowed: info.trade_allowed,
//             dlls_allowed: info.dlls_allowed,
//             terminal_connected: info.terminal_connected,
//             trade_expert: info.trade_expert,
//             margin_mode: info.margin_mode,
//             currency_digits: info.currency_digits,
//             fifo_close: info.fifo_close,
//             balance: info.balance,
//             credit: info.credit,
//             profit: info.profit,
//             equity: info.equity,
//             margin: info.margin,
//             margin_free: info.margin_free,
//             margin_level: info.margin_level,
//             margin_so_call: info.margin_so_call,
//             margin_so_so: info.margin_so_so,
//             margin_initial: info.margin_initial,
//             margin_maintenance: info.margin_maintenance,
//             assets: info.assets,
//             liabilities: info.liabilities,
//             commission_blocked: info.commission_blocked,
//             name: info.name,
//             server: info.server,
//             currency: info.currency,
//             company: info.company,
//             created_time: info.created_time,
//             updated_time: info.updated_time,
//         }
//     }
// }




