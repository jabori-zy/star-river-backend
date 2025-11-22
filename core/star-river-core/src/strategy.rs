use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    custom_type::{StrategyId, StrategyName},
    system::DateTimeUtc,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StrategyInfo {
    pub id: StrategyId,
    pub name: StrategyName,
    pub description: String,
    pub status: String,
    pub trade_mode: String,
    pub node_count: usize,
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub create_time: DateTimeUtc,
    #[schema(value_type = String, example = "2021-01-01 00:00:00")]
    pub update_time: DateTimeUtc,
}

impl StrategyInfo {
    pub fn new(
        id: StrategyId,
        name: StrategyName,
        description: String,
        status: String,
        trade_mode: String,
        node_count: usize,
        create_time: DateTimeUtc,
        update_time: DateTimeUtc,
    ) -> Self {
        Self {
            id,
            name,
            description,
            status,
            trade_mode,
            node_count,
            create_time,
            update_time,
        }
    }
}
