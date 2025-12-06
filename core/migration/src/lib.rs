// sea-orm-cli migrate refresh -d ./migration
pub use sea_orm_migration::prelude::*;
mod m20240101_000001_create_strategy_config_table; // Strategy configuration table
mod m20240101_000002_create_order_table; // Order table
mod m20240101_000003_create_position_table; // Position table
mod m20240101_000004_create_transaction_table; // Transaction table
mod m20240101_000005_create_account_config_table; // Account configuration table
// mod m20240101_000006_create_mt5_account_config_table; // MT5 account configuration table
// mod m20240101_000007_create_mt5_account_info_table; // MT5 account info table
mod m20240101_000008_create_account_info_table; // Account info table
// mod m20240101_000009_create_strategy_statistics_table; // Strategy statistics table
// mod m20240101_000010_create_strategy_sys_variable_table;
// mod m20250525_063811_add_test1_to_strateyg_config;
mod m20250610_024732_create_system_config_table;
mod m20250610_030125_insert_system_config_init_data;
// mod m20250907_101341_change_strategy_config_table_status_field;
// mod m20251117_063612_strategy_config_delete_config_field;
mod m20251203_014716_insert_exchange_init_data;
mod m20251205_095239_insert_demo_strategy;

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
            // Box::new(m20240101_000006_create_mt5_account_config_table::Migration),
            // Box::new(m20240101_000007_create_mt5_account_info_table::Migration),
            Box::new(m20240101_000008_create_account_info_table::Migration),
            // Box::new(m20240101_000009_create_strategy_statistics_table::Migration),
            // Box::new(m20240101_000010_create_strategy_sys_variable_table::Migration),
            // Box::new(m20250525_063811_add_test1_to_strateyg_config::Migration),
            Box::new(m20250610_024732_create_system_config_table::Migration),
            Box::new(m20250610_030125_insert_system_config_init_data::Migration),
            // Box::new(m20250907_101341_change_strategy_config_table_status_field::Migration),
            // Box::new(m20251117_063612_strategy_config_delete_config_field::Migration),
            Box::new(m20251203_014716_insert_exchange_init_data::Migration),
            Box::new(m20251205_095239_insert_demo_strategy::Migration),
        ]
    }
}
