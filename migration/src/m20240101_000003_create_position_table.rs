use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000003_create_position_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Position::Table)
                    .col(
                        ColumnDef::new(Position::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Position::StrategyId).big_integer().not_null())
                    .col(ColumnDef::new(Position::NodeId).string().not_null())
                    .col(ColumnDef::new(Position::ExchangePositionId).big_integer().not_null())
                    .col(ColumnDef::new(Position::AccountId).integer().not_null())
                    .col(ColumnDef::new(Position::Exchange).string().not_null())
                    .col(ColumnDef::new(Position::Symbol).string().not_null())
                    .col(ColumnDef::new(Position::PositionSide).string().not_null())
                    .col(ColumnDef::new(Position::PositionState).string().not_null())
                    .col(ColumnDef::new(Position::Quantity).double().not_null())
                    .col(ColumnDef::new(Position::OpenPrice).double().not_null())
                    .col(ColumnDef::new(Position::UnrealizedProfit).double())
                    .col(ColumnDef::new(Position::Sl).double())
                    .col(ColumnDef::new(Position::Tp).double())
                    .col(ColumnDef::new(Position::ExtraInfo).json())
                    .col(
                        ColumnDef::new(Position::CreatedTime)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Position::UpdatedTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .index(
                        Index::create()
                            .unique()
                            .name("idx_exchange_position_id_unique") // 唯一索引 交易所订单ID+交易所名称
                            .col(Position::Exchange)
                            .col(Position::ExchangePositionId),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Position::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Position {
    Table,
    Id,                 // 主键
    StrategyId,         // 策略ID
    ExchangePositionId, // 交易所订单ID
    NodeId,             // 节点ID
    AccountId,          // 账户ID
    Exchange,           // 交易所
    Symbol,             // 交易对
    PositionSide,       // 订单方向
    PositionState,      // 持仓状态
    Quantity,           // 数量
    OpenPrice,
    UnrealizedProfit, //未平仓利润
    Tp,
    Sl,
    ExtraInfo,   // 额外信息
    CreatedTime, //创建时间
    UpdatedTime, //更新时间
}
