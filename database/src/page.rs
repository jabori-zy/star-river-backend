use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(
    title = "PageResult",
    description = "PageResult",
    example = json!({
        "data": [],
        "total_items": 0,
        "items_per_page": 0,
        "current_page": 0,
        "total_pages": 0,
    })
)]
pub struct PageResult<T> {
    pub data: Vec<T>,
    pub total_items: u64,
    pub items_per_page: u64,
    pub current_page: u64,
    pub total_pages: u64,
}

impl<T> PageResult<T> {
    pub fn new(data: Vec<T>, total_items: u64, current_page: u64, items_per_page: u64, total_pages: u64) -> Self {
        Self {
            data,
            total_items,
            items_per_page,
            current_page,
            total_pages,
        }
    }
}
