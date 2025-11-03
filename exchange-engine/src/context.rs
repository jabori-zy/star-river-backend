mod event_handler;
mod binance_handler;
// mod mt5_handler;
mod regist_manage;

use star_river_core::custom_type::AccountId;
use engine_core::EngineBaseContext;
use std::collections::HashMap;
use sea_orm::DatabaseConnection;
use database::query::account_config_query::AccountConfigQuery;
use super::state_machine::ExchangeEngineAction;
use engine_core::context_trait::EngineContextTrait;
use super::exchanges::Exchange;
use exchange_core::state_machine::ExchangeRunState;
use crate::error::{ExchangeEngineError, ExchangeClientNotRegisteredSnafu};




#[derive(Debug)]
pub struct ExchangeEngineContext {
    pub base_context: EngineBaseContext<ExchangeEngineAction>,
    pub exchanges: HashMap<AccountId, Exchange>, // 交易所的账户id -> 交易所 每个交易所对应一个账户
    pub database: DatabaseConnection,
}

impl ExchangeEngineContext {
    pub fn new(
        base_context: EngineBaseContext<ExchangeEngineAction>,
        database: DatabaseConnection
    ) -> Self {
        Self {
            base_context,
            exchanges: HashMap::new(),
            database,
        }
    }
}



impl EngineContextTrait for ExchangeEngineContext {
    type Action = ExchangeEngineAction;

    fn base_context(&self) -> &EngineBaseContext<Self::Action> {
        &self.base_context
    }

    fn base_context_mut(&mut self) -> &mut EngineBaseContext<Self::Action> {
        &mut self.base_context
    }

}



impl ExchangeEngineContext {

    pub fn is_registered(&self, account_id: &i32) -> bool {
        self.exchanges.contains_key(account_id)
    }

    pub async fn get_exchange_instance(&self, account_id: &i32) -> Result<&Exchange, ExchangeEngineError> {
        match self.exchanges.get(account_id) {
            Some(client) => {
                Ok(client)
            }
            None => {
                let account_config = AccountConfigQuery::get_account_config_by_id(&self.database, *account_id).await?;
                Err(ExchangeClientNotRegisteredSnafu {
                    account_id: *account_id,
                    exchange_name: account_config.exchange.to_string(),
                }
                .build())
            }
        }
    }

    // 添加一个获取可变引用的方法
    // pub async fn get_exchange_mut<'a>(&'a mut self, account_id: &i32) -> Result<&'a mut Box<dyn ExchangeClientCore>, ExchangeEngineError> {
    //     match self.exchanges.get_mut(account_id) {
    //         Some(client) => Ok(client),
    //         None => {
    //             let account_config = AccountConfigQuery::get_account_config_by_id(&self.database, *account_id).await?;
    //             Err(ExchangeClientNotRegisteredSnafu {
    //                 account_id: *account_id,
    //                 exchange_name: account_config.exchange.to_string(),
    //             }
    //             .build())
    //         }
    //     }
    // }

    pub(super) async fn get_exchange_status(&self, account_id: &AccountId) -> ExchangeRunState {
        let exchange = self.get_exchange_instance(account_id).await;
        match exchange {
            Ok(exchange) => exchange.run_state().await,
            Err(_) => ExchangeRunState::NotRegistered,
        }
    }
}
