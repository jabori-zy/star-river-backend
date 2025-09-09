use axum::{
    routing::{delete, get, post, put},
    Router,
};

use crate::api::market_api::{get_support_kline_intervals, get_symbol_list};
use crate::star_river::StarRiver;

pub fn create_market_routes() -> Router<StarRiver> {
    Router::new()
        .route("/symbol_list/{account_id}", get(get_symbol_list))
        .route(
            "/support_kline_intervals/{account_id}",
            get(get_support_kline_intervals),
        )
}
