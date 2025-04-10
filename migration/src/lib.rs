
// sea-orm-cli migrate refresh -d ./migration
pub use sea_orm_migration::prelude::*;
mod m20240101_000001_create_strategy_info_table;
mod m20240101_000001_create_order_table;


pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_strategy_info_table::Migration),
            Box::new(m20240101_000001_create_order_table::Migration)
        ]
    }
}
