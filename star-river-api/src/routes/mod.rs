// 路由模块组织
// pub mod account_routes;
// pub mod exchange_routes;

// #[cfg(not())]
pub mod doc;
// 
// pub mod doc_paid;
// pub mod market_routes;
pub mod sse_routes;
pub mod strategy_routes;
pub mod system_routes;


// pub mod cache_routes;


use crate::star_river::StarRiver;
use axum::Router;
// #[cfg(not())]
use doc::ApiDoc;
// 
// use doc_paid::ApiDoc;
use utoipa::OpenApi;
use utoipa_scalar::{Scalar, Servable};

// 创建完整的应用路由
pub fn create_app_routes(star_river: StarRiver) -> Router {
    let router = Router::new()
        // 嵌套策略相关路由
        .nest("/api/v1/strategy", strategy_routes::create_strategy_routes())
        // .nest("/api/v1/strategy/live", strategy_routes::create_live_strategy_routes())
        .nest("/api/v1/strategy/backtest", strategy_routes::create_backtest_strategy_routes());
        // 嵌套账户相关路由
        // .nest("/api/v1/account", account_routes::create_account_routes())
        // 嵌套市场相关路由
        // .nest("/api/v1/market", market_routes::create_market_routes())
        // 嵌套交易所相关路由
        // .nest("/api/v1/exchange", exchange_routes::create_exchange_routes());

    // 条件性地添加缓存路由
    
    {
        // router = router.nest("/api/v1/cache", cache_routes::create_cache_routes());
    }

    router
        // 嵌套实时数据流路由
        .nest("/api/v1/sse", sse_routes::create_sse_routes())
        // 嵌套系统相关路由
        .nest("/api/v1/system", system_routes::create_system_routes())
        // WebSocket路由
        // .merge(websocket_routes::create_websocket_routes())
        .merge(create_docs_routes())
        .with_state(star_river)
}

// 创建文档路由
fn create_docs_routes() -> Router<StarRiver> {
    Router::new()
        // OpenAPI JSON endpoint
        .route("/api-docs/openapi.json", axum::routing::get(serve_openapi))
        // Scalar UI
        .merge(Scalar::with_url("/docs", ApiDoc::openapi()))
}

// 提供 OpenAPI JSON
async fn serve_openapi() -> impl axum::response::IntoResponse {
    axum::response::Json(ApiDoc::openapi())
}
