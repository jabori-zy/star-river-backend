
// sea-orm-cli migrate refresh -d ./migration
pub use sea_orm_migration::prelude::*;
mod m20240101_000001_create_strategy_config_table; // 策略配置表
mod m20240101_000002_create_order_table; // 订单表
mod m20240101_000003_create_position_table; // 持仓表
mod m20240101_000004_create_transaction_table; // 交易明细表
mod m20240101_000005_create_account_config_table; // 账户配置表
// mod m20240101_000006_create_mt5_account_config_table; // mt5账户配置表
// mod m20240101_000007_create_mt5_account_info_table; // mt5账户信息表
mod m20240101_000008_create_account_info_table; // 账户信息表
// mod m20240101_000009_create_strategy_statistics_table; // 策略统计表
mod m20240101_000010_create_strategy_sys_variable_table;
// mod m20250525_063811_add_test1_to_strateyg_config;
mod m20250610_024732_create_system_config_table;
mod m20250610_030125_insert_system_config_init_data;



pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_strategy_config_table::Migration),
            Box::new(m20240101_000002_create_order_table::Migration),
            Box::new(m20240101_000003_create_position_table::Migration),
            Box::new(m20240101_000004_create_transaction_table::Migration),
            Box::new(m20240101_000005_create_account_config_table::Migration),
            Box::new(m20240101_000008_create_account_info_table::Migration),
            // Box::new(m20240101_000009_create_strategy_statistics_table::Migration),
            Box::new(m20240101_000010_create_strategy_sys_variable_table::Migration),
            // Box::new(m20250525_063811_add_test1_to_strateyg_config::Migration),
            Box::new(m20250610_024732_create_system_config_table::Migration),
            Box::new(m20250610_030125_insert_system_config_init_data::Migration),
        ]
    }
}
