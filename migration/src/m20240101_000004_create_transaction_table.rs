use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000004_create_transaction_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Transaction::Table)
                    .col(ColumnDef::new(Transaction::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Transaction::StrategyId).big_integer().not_null())
                    .col(ColumnDef::new(Transaction::NodeId).string().not_null())
                    .col(ColumnDef::new(Transaction::Exchange).string().not_null())
                    .col(ColumnDef::new(Transaction::Symbol).string().not_null())
                    .col(ColumnDef::new(Transaction::ExchangePositionId).big_integer().not_null())
                    .col(ColumnDef::new(Transaction::ExchangeTransactionId).big_integer().not_null())
                    .col(ColumnDef::new(Transaction::ExchangeOrderId).big_integer().not_null())
                    .col(ColumnDef::new(Transaction::TransactionType).string().not_null())
                    .col(ColumnDef::new(Transaction::TransactionSide).string().not_null())
                    .col(ColumnDef::new(Transaction::Quantity).double().not_null())
                    .col(ColumnDef::new(Transaction::Price).double().not_null())
                    .col(
                        ColumnDef::new(Transaction::CreateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(ColumnDef::new(Transaction::ExtraInfo).json())
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Transaction::Table).to_owned()).await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Transaction {
    Table,
    Id,                    // 主键
    StrategyId,            // 策略ID
    NodeId,                // 节点ID
    Exchange,              // 交易所
    Symbol,                // 交易对
    ExchangeTransactionId, // 每个交易所的交易明细ID
    ExchangePositionId,    // 持仓ID
    ExchangeOrderId,       // 订单ID
    TransactionType,       // 交易类型
    TransactionSide,       // 交易方向
    Quantity,              // 数量
    Price,                 // 价格
    CreateTime,           //创建时间
    ExtraInfo,             // 额外信息
}
