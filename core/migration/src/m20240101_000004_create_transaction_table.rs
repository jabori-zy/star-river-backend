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
    Id,                    // Primary key
    StrategyId,            // Strategy ID
    NodeId,                // Node ID
    Exchange,              // Exchange
    Symbol,                // Trading pair
    ExchangeTransactionId, // Exchange transaction ID
    ExchangePositionId,    // Position ID
    ExchangeOrderId,       // Order ID
    TransactionType,       // Transaction type
    TransactionSide,       // Transaction side
    Quantity,              // Quantity
    Price,                 // Price
    CreateTime,            // Created time
    ExtraInfo,             // Extra info
}
