use star_river_core::account::mt5_account::OriginalMt5AccountInfo;
use star_river_core::account::mt5_account::Mt5AccountInfo;
use crate::entities::mt5_account_info::Model as Mt5AccountInfoModel;
use crate::entities::mt5_account_info;
use sea_orm::*;
use chrono::Utc;

pub struct Mt5AccountInfoMutation;

impl Mt5AccountInfoMutation {
    pub async fn insert_mt5_account_info(
        db: &DbConn,
        original_account_info: OriginalMt5AccountInfo,
    ) -> Result<Mt5AccountInfo, DbErr> {
        let account_info_model = mt5_account_info::ActiveModel {
            id: NotSet,
            account_id: Set(original_account_info.account_id),
            login: Set(original_account_info.login),
            trade_mode: Set(original_account_info.trade_mode),
            leverage: Set(original_account_info.leverage),
            limit_orders: Set(original_account_info.limit_orders),
            margin_stopout_mode: Set(original_account_info.margin_stopout_mode),
            trade_allowed: Set(original_account_info.trade_allowed),
            dlls_allowed: Set(original_account_info.dlls_allowed),
            terminal_connected: Set(original_account_info.terminal_connected),
            trade_expert: Set(original_account_info.trade_expert),
            margin_mode: Set(original_account_info.margin_mode),
            currency_digits: Set(original_account_info.currency_digits),
            fifo_close: Set(original_account_info.fifo_close),
            balance: Set(original_account_info.balance),
            credit: Set(original_account_info.credit),
            profit: Set(original_account_info.profit),
            equity: Set(original_account_info.equity),
            margin: Set(original_account_info.margin),
            margin_free: Set(original_account_info.margin_free),
            margin_level: Set(original_account_info.margin_level),
            margin_so_call: Set(original_account_info.margin_so_call),
            margin_so_so: Set(original_account_info.margin_so_so),
            margin_initial: Set(original_account_info.margin_initial),
            margin_maintenance: Set(original_account_info.margin_maintenance),
            assets: Set(original_account_info.assets),
            liabilities: Set(original_account_info.liabilities),
            commission_blocked: Set(original_account_info.commission_blocked),
            name: Set(original_account_info.name),
            server: Set(original_account_info.server),
            currency: Set(original_account_info.currency),
            company: Set(original_account_info.company),
            created_time: Set(Utc::now()),
            updated_time: Set(Utc::now()),
        }.insert(db).await?;
        Ok(account_info_model.into())
    }

    pub async fn update_mt5_account_info(
        db: &DbConn,
        original_account_info: OriginalMt5AccountInfo,
    ) -> Result<Mt5AccountInfo, DbErr> {
        // 查找是否存在该账户的信息记录
        let account_info_model_option = mt5_account_info::Entity::find()
            .filter(mt5_account_info::Column::AccountId.eq(original_account_info.account_id))
            .one(db)
            .await?;

        // 根据查找结果决定是更新还是新增
        match account_info_model_option {
            // 存在记录，执行更新操作
            Some(existing_model) => {
                let account_info_model = mt5_account_info::ActiveModel {
                    id: Set(existing_model.id),
                    account_id: Set(original_account_info.account_id),
                    login: Set(original_account_info.login),
                    trade_mode: Set(original_account_info.trade_mode),
                    leverage: Set(original_account_info.leverage),
                    limit_orders: Set(original_account_info.limit_orders),
                    margin_stopout_mode: Set(original_account_info.margin_stopout_mode),
                    trade_allowed: Set(original_account_info.trade_allowed),
                    dlls_allowed: Set(original_account_info.dlls_allowed),
                    terminal_connected: Set(original_account_info.terminal_connected),
                    trade_expert: Set(original_account_info.trade_expert),
                    margin_mode: Set(original_account_info.margin_mode),
                    currency_digits: Set(original_account_info.currency_digits),
                    fifo_close: Set(original_account_info.fifo_close),
                    balance: Set(original_account_info.balance),
                    credit: Set(original_account_info.credit),
                    profit: Set(original_account_info.profit),
                    equity: Set(original_account_info.equity),
                    margin: Set(original_account_info.margin),
                    margin_free: Set(original_account_info.margin_free),
                    margin_level: Set(original_account_info.margin_level),
                    margin_so_call: Set(original_account_info.margin_so_call),
                    margin_so_so: Set(original_account_info.margin_so_so),
                    margin_initial: Set(original_account_info.margin_initial),
                    margin_maintenance: Set(original_account_info.margin_maintenance),
                    assets: Set(original_account_info.assets),
                    liabilities: Set(original_account_info.liabilities),
                    commission_blocked: Set(original_account_info.commission_blocked),
                    name: Set(original_account_info.name),
                    server: Set(original_account_info.server),
                    currency: Set(original_account_info.currency),
                    company: Set(original_account_info.company),
                    updated_time: Set(Utc::now()),
                    ..Default::default()
                }
                .update(db)
                .await?;
                
                Ok(account_info_model.into())
            },
            
            // 不存在记录，执行新增操作
            None => {
                let account_info_model = Mt5AccountInfoMutation::insert_mt5_account_info(db, original_account_info).await?;
                Ok(account_info_model)
            }
        }
    }
}