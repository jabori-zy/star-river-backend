
// sea-orm-cli migrate refresh -d ./migration
pub use sea_orm_migration::prelude::*;
mod m20240101_000001_create_strategy_info_table; // 策略信息表
mod m20240101_000002_create_order_table; // 订单表
mod m20240101_000003_create_position_table; // 持仓表
mod m20240101_000004_create_transaction_detail_table; // 交易明细表
mod m20240101_000005_create_account_config_table; // 账户配置表

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_strategy_info_table::Migration),
            Box::new(m20240101_000002_create_order_table::Migration),
            Box::new(m20240101_000003_create_position_table::Migration),
            Box::new(m20240101_000004_create_transaction_detail_table::Migration),
            Box::new(m20240101_000005_create_account_config_table::Migration),
        ]
    }
}
