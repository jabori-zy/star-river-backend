use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000002_create_order_table" // Make sure this matches with the file name
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
                    .col(ColumnDef::new(Order::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(Order::StrategyId).big_integer().not_null())
                    .col(ColumnDef::new(Order::NodeId).string().not_null())
                    .col(ColumnDef::new(Order::ExchangeOrderId).big_integer().not_null())
                    .col(ColumnDef::new(Order::AccountId).integer().not_null())
                    .col(ColumnDef::new(Order::Exchange).string().not_null())
                    .col(ColumnDef::new(Order::Symbol).string().not_null())
                    .col(ColumnDef::new(Order::OrderSide).string().not_null())
                    .col(ColumnDef::new(Order::OrderStatus).string().not_null())
                    .col(ColumnDef::new(Order::OrderType).string().not_null())
                    .col(ColumnDef::new(Order::Quantity).double().not_null())
                    .col(ColumnDef::new(Order::Price).double().not_null())
                    .col(ColumnDef::new(Order::Sl).double())
                    .col(ColumnDef::new(Order::Tp).double())
                    .col(ColumnDef::new(Order::ExtraInfo).json())
                    .col(
                        ColumnDef::new(Order::CreateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
                    .col(
                        ColumnDef::new(Order::UpdateTime)
                            .timestamp()
                            .not_null()
                            .default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())),
                    )
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
        manager.drop_table(Table::drop().table(Order::Table).to_owned()).await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Order {
    Table,
    Id,              // Primary key
    StrategyId,      // Strategy ID
    ExchangeOrderId, // Exchange order ID
    NodeId,          // Node ID
    AccountId,       // Account ID
    Exchange,        // Exchange
    Symbol,          // Trading pair
    OrderSide,       // Order side
    OrderType,       // Order type
    OrderStatus,     // Order status
    Quantity,        // Quantity
    Price,           // Price
    Tp,              // Take profit
    Sl,              // Stop loss
    ExtraInfo,       // Extra info
    CreateTime,      // Created time
    UpdateTime,      // Updated time
}
