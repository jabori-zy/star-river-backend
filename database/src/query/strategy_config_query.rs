use sea_orm::*;
use ::entity::{strategy_config, strategy_config::Entity as StrategyConfig};
use types::strategy::Strategy;

pub struct StrategyConfigQuery;


impl StrategyConfigQuery {

    // 分页查询策略列表
    pub async fn get_strategy_list_in_page(
        db: &DbConn,
        page: u64,
        strategy_per_page: u64
    ) -> Result<(Vec<Strategy>, u64), DbErr> {
        let paginator = StrategyConfig::find()
        // 只查询未删除的策略
        .filter(strategy_config::Column::IsDeleted.eq(false))
        .order_by_desc(strategy_config::Column::CreatedTime)
        .paginate(db, strategy_per_page);
        
        let num_pages = paginator.num_pages().await?;

        paginator.fetch_page(page - 1).await.map(|p| (p.into_iter().map(|config| config.into()).collect(), num_pages))
    }

    // 获取所有未删除的策略
    pub async fn get_all_strategy(
        db: &DbConn,
    ) -> Result<Vec<Strategy>, DbErr> {
        let strategies = StrategyConfig::find().filter(strategy_config::Column::IsDeleted.eq(false)).all(db).await?;
        Ok(strategies.into_iter().map(|config| config.into()).collect())
    }

    pub async fn get_strategy_by_id(
        db: &DbConn,
        id: i32
    ) -> Result<Option<Strategy>, DbErr> {
        let strategy_config = StrategyConfig::find_by_id(id)
        .filter(strategy_config::Column::IsDeleted.eq(false))
        .one(db).await?;
        Ok(strategy_config.map(|config| config.into()))
    }

    pub async fn get_backtest_chart_config_by_strategy_id(
        db: &DbConn,
        strategy_id: i32
    ) -> Result<JsonValue, DbErr> {
        let strategy_config = StrategyConfig::find_by_id(strategy_id)
        .filter(strategy_config::Column::IsDeleted.eq(false))
        .one(db).await?;
    
        if let Some(config) = strategy_config {
            Ok(config.backtest_chart_config.unwrap_or(JsonValue::Null))
        } else {
            Ok(JsonValue::Null)
        }
    }
}

