use ::entity::{strategy_config, strategy_config::Entity as StrategyConfigEntity};
use sea_orm::*;
use star_river_core::strategy::StrategyInfo;
use strategy_core::strategy::StrategyConfig;

use crate::{error::DatabaseError, page::PageResult};

pub struct StrategyConfigQuery;

impl StrategyConfigQuery {
    // Get paginated strategy list
    pub async fn get_strategy_list_in_page(
        db: &DbConn,
        page: u64,
        strategy_per_page: u64,
    ) -> Result<PageResult<StrategyInfo>, DatabaseError> {
        let paginator = StrategyConfigEntity::find()
            // Filter undeleted strategies
            .filter(strategy_config::Column::IsDeleted.eq(false))
            .order_by_desc(strategy_config::Column::CreateTime)
            .paginate(db, strategy_per_page);

        // Query total_items once and calculate total_pages to avoid duplicate COUNT(*) queries
        let total_items = paginator.num_items().await?;
        let total_pages = (total_items + strategy_per_page - 1) / strategy_per_page;

        let models = paginator.fetch_page(page - 1).await?;
        let strategy_infos = models
            .into_iter()
            .map(|m| {
                let node_count = if let Some(n) = m.nodes
                    && let Some(arr) = n.as_array()
                {
                    arr.len()
                } else {
                    0
                };

                StrategyInfo::new(
                    m.id,
                    m.name,
                    m.description,
                    m.status,
                    m.trade_mode,
                    node_count,
                    m.create_time,
                    m.update_time,
                )
            })
            .collect::<Vec<StrategyInfo>>();
        let page_result = PageResult::new(strategy_infos, total_items, page, strategy_per_page, total_pages);

        Ok(page_result)
    }

    // Get all undeleted strategies
    pub async fn get_all_strategy(db: &DbConn) -> Result<Vec<StrategyConfig>, DbErr> {
        let strategy_models = StrategyConfigEntity::find()
            .filter(strategy_config::Column::IsDeleted.eq(false))
            .all(db)
            .await?;

        let mut strategy_configs = Vec::new();
        for model in strategy_models {
            let strategy_config = model.into();
            strategy_configs.push(strategy_config);
        }

        Ok(strategy_configs)
    }

    pub async fn get_strategy_by_id(db: &DbConn, id: i32) -> Result<StrategyConfig, DatabaseError> {
        let strategy_model = StrategyConfigEntity::find_by_id(id)
            .filter(strategy_config::Column::IsDeleted.eq(false))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find strategy config.".to_owned()))?;

        Ok(strategy_model.into())
    }

    pub async fn get_backtest_chart_config_by_strategy_id(db: &DbConn, strategy_id: i32) -> Result<JsonValue, DatabaseError> {
        let strategy_config = StrategyConfigEntity::find_by_id(strategy_id)
            .filter(strategy_config::Column::IsDeleted.eq(false))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find strategy.".to_owned()))?;

        Ok(strategy_config.backtest_chart_config.unwrap_or(JsonValue::Null))
    }

    pub async fn get_strategy_run_state(db: &DbConn, strategy_id: i32) -> Result<String, DatabaseError> {
        let strategy_config = StrategyConfigEntity::find_by_id(strategy_id)
            .filter(strategy_config::Column::IsDeleted.eq(false))
            .one(db)
            .await?
            .ok_or(DbErr::RecordNotFound("Cannot find strategy.".to_owned()))?;

        Ok(strategy_config.status)
    }
}
