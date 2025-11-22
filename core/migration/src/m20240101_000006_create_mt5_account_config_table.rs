use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m_20240101_000006_create_mt5_account_config_table"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    // Define how to apply this migration: Create the Chef table.
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Mt5AccountConfig::Table)
                    .col(
                        ColumnDef::new(Mt5AccountConfig::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Mt5AccountConfig::AccountName).string().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::Exchange).string().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::IsAvailable).boolean().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::Login).big_integer().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::Password).string().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::Server).string().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::TerminalPath).string().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::IsDeleted).boolean().not_null().default(false))
                    .col(ColumnDef::new(Mt5AccountConfig::SortIndex).integer().not_null())
                    .col(ColumnDef::new(Mt5AccountConfig::CreateTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .col(ColumnDef::new(Mt5AccountConfig::UpdateTime).timestamp().not_null().default(SimpleExpr::Custom("CURRENT_TIMESTAMP".to_string())))
                    .index(
                        Index::create()
                        .unique()
                        .name("idx_login_terminal_path_unique")
                        .col(Mt5AccountConfig::Login)
                        .col(Mt5AccountConfig::TerminalPath)
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the Chef table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Mt5AccountConfig::Table).to_owned())
            .await
    }
}

// For ease of access
#[derive(Iden)]
pub enum Mt5AccountConfig {
    Table,
    Id, // 主键
    AccountName, // 账户名称
    Exchange, // 交易所
    IsAvailable, // 是否可用
    Login, // 账户ID
    Password, // 密码
    Server, // 服务器
    TerminalPath, // 终端路径
    SortIndex, // 排序索引
    IsDeleted, // 是否删除
    CreateTime,//创建时间
    UpdateTime,//更新时间
}