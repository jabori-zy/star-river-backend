use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000003_create_order_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Order::Table)
                    .col(
                        ColumnDef::new(Order::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Order::OrderId).integer().not_null())
                    .col(ColumnDef::new(Order::StrategyId).integer().not_null())
                    .col(ColumnDef::new(Order::Exchange).string().not_null())
                    .col(ColumnDef::new(Order::Symbol).string().not_null())
                    .col(ColumnDef::new(Order::OrderSide).string().not_null())
                    .col(ColumnDef::new(Order::OrderStatus).string().not_null())
                    .col(ColumnDef::new(Order::OrderType).string().not_null())
                    .col(ColumnDef::new(Order::Quantity).float().not_null())
                    .col(ColumnDef::new(Order::Price).float().not_null())
                    .col(ColumnDef::new(Order::Sl).float())
                    .col(ColumnDef::new(Order::Tp).float())
                    .col(ColumnDef::new(Order::CreatedTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .col(ColumnDef::new(Order::UpdatedTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
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
            .drop_table(Table::drop().table(Order::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Order {
    Table,
    Id,
    OrderId,
    StrategyId,
    Exchange,
    Symbol,
    OrderSide,
    OrderType,
    OrderStatus,
    Quantity,
    Price,
    Tp,
    Sl,
    CreatedTime,//创建时间
    UpdatedTime,//更新时间
}