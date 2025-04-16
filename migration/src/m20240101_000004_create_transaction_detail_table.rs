use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000004_create_transaction_detail_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TransactionDetail::Table)
                    .col(
                        ColumnDef::new(TransactionDetail::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(TransactionDetail::StrategyId).big_integer().not_null())
                    .col(ColumnDef::new(TransactionDetail::NodeId).string().not_null())
                    .col(ColumnDef::new(TransactionDetail::Exchange).string().not_null())
                    .col(ColumnDef::new(TransactionDetail::Symbol).string().not_null())
                    .col(ColumnDef::new(TransactionDetail::ExchangePositionId).big_integer().not_null())
                    .col(ColumnDef::new(TransactionDetail::ExchangeTransactionId).big_integer().not_null())
                    .col(ColumnDef::new(TransactionDetail::ExchangeOrderId).big_integer().not_null())
                    .col(ColumnDef::new(TransactionDetail::TransactionType).string().not_null())
                    .col(ColumnDef::new(TransactionDetail::TransactionSide).string().not_null())
                    .col(ColumnDef::new(TransactionDetail::Quantity).double().not_null())
                    .col(ColumnDef::new(TransactionDetail::Price).double().not_null())
                    .col(ColumnDef::new(TransactionDetail::CreatedTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    // .col(ColumnDef::new(Position::UpdatedTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    // .foreign_key(
                    //     ForeignKey::create()
                    //         .name("fk-strategy_info-bakery_id")
                    //         .from(StrategyInfo::Table, StrategyInfo::BakeryId)
                    //         .to(Bakery::Table, Bakery::Id),
                    // )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TransactionDetail::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum TransactionDetail {
    Table,
    Id, // 主键
    StrategyId, // 策略ID
    NodeId, // 节点ID
    Exchange, // 交易所
    Symbol, // 交易对
    ExchangeTransactionId, // 每个交易所的交易明细ID
    ExchangePositionId, // 持仓ID
    ExchangeOrderId, // 订单ID
    TransactionType, // 交易类型
    TransactionSide, // 交易方向
    Quantity, // 数量
    Price, // 价格
    CreatedTime,//创建时间
}