use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000007_create_mt5_account_info_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Mt5AccountInfo::Table)
                    .col(
                        ColumnDef::new(Mt5AccountInfo::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Mt5AccountInfo::AccountId).integer().not_null()) //账户配置id
                    .col(ColumnDef::new(Mt5AccountInfo::Login).big_integer().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::TradeMode).string().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Leverage).big_integer().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::LimitOrders).big_integer().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginStopoutMode).string().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::TradeAllowed).boolean().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::DllsAllowed).boolean().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::TerminalConnected).boolean().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::TradeExpert).boolean().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginMode).string().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::CurrencyDigits).big_integer().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::FifoClose).boolean().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Balance).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Credit).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Profit).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Equity).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Margin).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginFree).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginLevel).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginSoCall).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginSoSo).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginInitial).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::MarginMaintenance).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Assets).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Liabilities).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::CommissionBlocked).double().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Name).string().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Server).string().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Currency).string().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::Company).string().not_null())
                    .col(ColumnDef::new(Mt5AccountInfo::CreateTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .col(ColumnDef::new(Mt5AccountInfo::UpdateTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mt5AccountInfo::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Mt5AccountInfo {
    Table,
    Id,
    AccountId,
    Login,
    TradeMode,
    Leverage,
    LimitOrders,
    MarginStopoutMode,
    TradeAllowed,
    DllsAllowed,
    TerminalConnected,
    TradeExpert,
    MarginMode,
    CurrencyDigits,
    FifoClose,
    Balance,
    Credit,
    Profit,
    Equity,
    Margin,
    MarginFree,
    MarginLevel,
    MarginSoCall,
    MarginSoSo,
    MarginInitial,
    MarginMaintenance,
    Assets,
    Liabilities,
    CommissionBlocked,
    Name,
    Server,
    Currency,
    Company,
    CreateTime,
    UpdateTime,
}