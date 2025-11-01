mod server;

use star_river::{routes::create_app_routes, star_river::StarRiver, star_river::init_app};
use axum::extract::State;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    server::init_logging()?;

    // 创建 CORS 配置
    let cors = server::create_cors();

    // 创建应用状态
    let state = StarRiver::new().await;

    // 创建路由
    let app = create_app_routes(state.clone()).layer(cors);

    // 初始化应用
    init_app(State(state)).await;

    // 获取监听地址
    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3100".to_string())
        .parse()?;

    // 启动服务器
    server::serve(app, addr).await
}
