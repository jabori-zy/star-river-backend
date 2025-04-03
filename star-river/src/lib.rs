pub mod star_river;
pub mod api;
pub mod websocket;
pub mod sse;


use axum::{routing::{get, post, delete}, Router, routing::any};
use axum::extract::State;

use std::net::SocketAddr;
use tokio;
use tower_http::cors::{Any, CorsLayer};
use axum::http::HeaderValue;
use crate::star_river::StarRiver;
use crate::api::mutation_api::{create_strategy, update_strategy, delete_strategy};
use crate::api::query_api::{get_strategy_list, get_strategy_by_id};
use crate::api::strategy_api::{run_strategy, stop_strategy, init_strategy, enable_strategy_event_push, disable_strategy_event_push};
use crate::sse::{market_sse_handler, indicator_sse_handler, strategy_sse_handler};
use tracing::Level;
use crate::websocket::ws_handler;
use crate::star_river::init_app;

#[tokio::main]
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // 设置生产环境的日志级别
    tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::DEBUG)
        // build but do not install the subscriber.
        .init();

    // build our application with a route
    // 设置跨域
    let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods(Any)
    .allow_headers(Any);

    // 创建app状态
    let star_river = StarRiver::new().await;

    let app = Router::new()
        .route("/", get(hello_world))
        .route("/ws", any(ws_handler))
        .route("/market_sse", get(market_sse_handler))
        .route("/indicator_sse", get(indicator_sse_handler))
        .route("/strategy_sse", get(strategy_sse_handler))
        .route("/create_strategy", post(create_strategy))
        .route("/init_strategy", post(init_strategy))
        .route("/run_strategy", post(run_strategy))
        .route("/stop_strategy", post(stop_strategy))
        .route("/enable_strategy_event_push", post(enable_strategy_event_push))
        .route("/disable_strategy_event_push", post(disable_strategy_event_push))
        .route("/get_strategy_list", get(get_strategy_list))
        .route("/update_strategy", post(update_strategy))
        .route("/delete_strategy", delete(delete_strategy))
        .route("/get_strategy", get(get_strategy_by_id))
        .layer(cors)
        .with_state(star_river.clone());

    // 初始化app
    init_app(State(star_river)).await;

    


    // 允许从环境变量配置监听地址
    let addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3100".to_string())
        .parse::<SocketAddr>()
        .expect("Invalid server address");

    // run it
    // let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let listener = bind_with_retry(addr, 3).await?;
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();
    
    let server = axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>());
    let graceful = server.with_graceful_shutdown(async {
        rx.await.ok();
        
        // 使用 timeout 包装关闭流程
        if let Err(_) = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            async {
                #[cfg(windows)]
                {
                    tracing::info!("正在清理 MetaTrader5 进程...");
                    let _ = std::process::Command::new("taskkill")
                        .args(&["/F", "/IM", "MetaTrader5.exe"])
                        .output();
                    
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    tracing::info!("清理完成，继续关闭服务器...");
                }
            }
        ).await {
            tracing::warn!("关闭流程超时，强制退出");
            std::process::exit(0);
        }
    });

    tokio::spawn(async move {
        if let Ok(()) = tokio::signal::ctrl_c().await {
            tracing::info!("接收到关闭信号，正在优雅关闭...");
            let _ = tx.send(());
        }
    });

    if let Err(e) = graceful.await {
        tracing::error!("服务器错误: {}", e);
    }

    Ok(())
}





async fn hello_world() -> String {
    tracing::info!("hello_world");
    "Hello, World!".to_string()
}

async fn bind_with_retry(addr: SocketAddr, max_retries: u32) -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                if retries >= max_retries {
                    return Err(format!("端口 {} 被占用，重试 {} 次后仍然失败", addr.port(), max_retries).into());
                }
                tracing::warn!("端口 {} 被占用，尝试清理 MetaTrader5 进程...", addr.port());
                
                #[cfg(windows)]
                {
                    // 查找并清理 MetaTrader5 进程
                    let output = std::process::Command::new("tasklist")
                        .args(&["/FI", "IMAGENAME eq MetaTrader5.exe", "/FO", "CSV"])
                        .output()?;
                    
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.contains("MetaTrader5.exe") {
                        tracing::warn!("发现旧的MetaTrader5进程, 正在清理...");
                        
                        // 强制结束所有MetaTrader5.exe进程
                        let kill_result = std::process::Command::new("taskkill")
                            .args(&["/F", "/IM", "MetaTrader5.exe"])
                            .output();
                            
                        match kill_result {
                            Ok(_) => tracing::info!("成功清理 MetaTrader5 进程"),
                            Err(e) => tracing::warn!("清理 MetaTrader5 进程失败: {}", e),
                        }
                    }
                }
                
                // 等待进程完全退出
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                retries += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
}






