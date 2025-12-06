// Route module organization
pub mod account_routes;
pub mod exchange_routes;

// #[cfg(not())]
pub mod doc;
//
// pub mod doc_paid;
pub mod market_routes;
pub mod sse_routes;
pub mod strategy_routes;
pub mod system_routes;

// pub mod cache_routes;

use axum::Router;
// #[cfg(not())]
use doc::ApiDoc;
//
// use doc_paid::ApiDoc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

use crate::star_river::StarRiver;

// Create complete application routes
pub fn create_app_routes(star_river: StarRiver) -> Router {
    let router = Router::new()
        // Nest strategy related routes
        .nest("/api/v1/strategy", strategy_routes::create_strategy_routes())
        // .nest("/api/v1/strategy/live", strategy_routes::create_live_strategy_routes())
        .nest("/api/v1/strategy/backtest", strategy_routes::create_backtest_strategy_routes())
        // Nest account related routes
        .nest("/api/v1/account", account_routes::create_account_routes())
        // Nest market related routes
        .nest("/api/v1/market", market_routes::create_market_routes())
        // Nest exchange related routes
        .nest("/api/v1/exchange", exchange_routes::create_exchange_routes());

    // Conditionally add cache routes

    {
        // router = router.nest("/api/v1/cache", cache_routes::create_cache_routes());
    }

    router
        // Nest real-time data stream routes
        .nest("/api/v1/sse", sse_routes::create_sse_routes())
        // Nest system related routes
        .nest("/api/v1/system", system_routes::create_system_routes())
        // WebSocket routes
        // .merge(websocket_routes::create_websocket_routes())
        .merge(create_docs_routes())
        .with_state(star_river)
}

// Create documentation routes
fn create_docs_routes() -> Router<StarRiver> {
    Router::new()
        // OpenAPI JSON endpoint
        .route("/api-docs/openapi.json", axum::routing::get(serve_openapi))
        // Scalar UI
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}

// Serve OpenAPI JSON
async fn serve_openapi() -> impl axum::response::IntoResponse {
    axum::response::Json(ApiDoc::openapi())
}
