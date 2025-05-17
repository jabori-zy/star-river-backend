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
use crate::api::mutation_api::account_mutation::{add_account_config, delete_account_config, update_account_config, update_account_config_is_available};
use crate::api::query_api::{get_strategy_list, get_strategy_by_id, get_account_config};
use crate::api::strategy_api::{run_strategy, stop_strategy, init_strategy, get_strategy_cache_keys, enable_strategy_data_push, disable_strategy_data_push};
use crate::api::cache_api::{get_cache_key, get_memory_size, get_cache_value};
use crate::sse::{market_sse_handler, indicator_sse_handler, strategy_sse_handler, account_sse_handler};
use crate::api::account_api::login_mt5_account;
use tracing::{Level, instrument};
use crate::websocket::ws_handler;
use crate::star_river::init_app;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::layer;
use tracing_subscriber::fmt::time::{OffsetTime};
use time::UtcOffset;
use time::macros::format_description;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_appender::non_blocking::NonBlocking;
use std::path::Path;
use std::fs;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, Registry};
use tracing_subscriber::fmt::format;

#[tokio::main]
pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    // 设置生产环境的日志级别
    // tracing_subscriber::fmt()
    //     // filter spans/events with level TRACE or higher.
    //     .with_max_level(Level::DEBUG)
    //     // build but do not install the subscriber.
    //     .init();
    // 确保log目录存在
    let log_dir = Path::new("logs");
    if !log_dir.exists() {
        fs::create_dir_all(log_dir)?;
    }
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        log_dir,
        "star-river.log"
    );
    // 处理非阻塞appender
    let (non_blocking_appender, _guard) = tracing_appender::non_blocking(file_appender);
    let stdout = std::io::stdout.with_max_level(tracing::Level::DEBUG);
    let filter = EnvFilter::new("debug,hyper=error,hyper_util=error,reqwest=error");
    
    // 设置为UTC+8时区（北京时间）
    let offset = UtcOffset::current_local_offset().expect("should get local offset!");
    let time_format = format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:6]"
    );
    let timer = OffsetTime::new(offset, time_format);
    let console_layer = layer()
    .with_writer(stdout)
    .with_ansi(true) // 控制台保留ANSI颜色
    .with_timer(timer.clone());

    let file_layer = layer()
    .with_writer(non_blocking_appender)
    .with_ansi(false) // 文件中不使用ANSI颜色
    .with_timer(timer.clone());

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer) // 文件输出放到控制台输出的上方。不然文件中会有乱码
        .with(console_layer)
        .init();

    // tracing_subscriber::fmt()
    //     // filter spans/events with level TRACE or higher.
    //     .with_max_level(Level::DEBUG)
    //     .with_env_filter(filter)
    //     .with_timer(timer)
    //     .with_writer(stdout.and(file_appender))
    //     // build but do not install the subscriber.
    //     .init();

    // build our application with a route
    // 设置跨域
    let cors = CorsLayer::new()
    .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
    .allow_methods(Any)
    .allow_headers(Any);

    // 创建app状态
    let star_river = StarRiver::new().await;

    let app = Router::new()
        .route("/ws", any(ws_handler))
        .route("/market_sse", get(market_sse_handler))
        .route("/indicator_sse", get(indicator_sse_handler))
        .route("/strategy_sse", get(strategy_sse_handler))
        .route("/account_sse", get(account_sse_handler))
        .route("/create_strategy", post(create_strategy))
        .route("/init_strategy", post(init_strategy))
        .route("/run_strategy", post(run_strategy))
        .route("/stop_strategy", post(stop_strategy))
        .route("/get_strategy_list", get(get_strategy_list))
        .route("/update_strategy", post(update_strategy))
        .route("/delete_strategy", delete(delete_strategy))
        .route("/get_strategy", get(get_strategy_by_id))
        .route("/add_account_config", post(add_account_config))
        .route("/get_account_config", get(get_account_config))
        .route("/delete_account_config", delete(delete_account_config))
        .route("/update_account_config", post(update_account_config))
        .route("/update_account_config_is_available", post(update_account_config_is_available))
        .route("/login_mt5_account", post(login_mt5_account))
        .route("/get_cache_key", get(get_cache_key))
        .route("/get_memory_size", get(get_memory_size))
        .route("/get_strategy_cache_keys", get(get_strategy_cache_keys))
        .route("/get_cache_value", get(get_cache_value))
        .route("/enable_strategy_data_push", post(enable_strategy_data_push))
        .route("/disable_strategy_data_push", post(disable_strategy_data_push))
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

async fn bind_with_retry(addr: SocketAddr, max_retries: u32) -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
    let mut retries = 0;
    loop {
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => return Ok(listener),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                if retries >= max_retries {
                    return Err(format!("端口 {} 被占用，重试 {} 次后仍然失败", addr.port(), max_retries).into());
                }
                tracing::warn!("端口 {} 被占用，尝试清理所有 MetaTrader5 相关进程...", addr.port());
                
                #[cfg(windows)]
                {
                    // 1. 首先检查并清理原始的 MetaTrader5.exe 进程
                    let output = std::process::Command::new("tasklist")
                        .args(&["/FI", "IMAGENAME eq MetaTrader5.exe", "/FO", "CSV"])
                        .output()?;
                    
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.contains("MetaTrader5.exe") {
                        tracing::warn!("发现旧的MetaTrader5.exe进程, 正在清理...");
                        
                        let kill_result = std::process::Command::new("taskkill")
                            .args(&["/F", "/IM", "MetaTrader5.exe"])
                            .output();
                            
                        match kill_result {
                            Ok(_) => tracing::info!("成功清理 MetaTrader5.exe 进程"),
                            Err(e) => tracing::warn!("清理 MetaTrader5.exe 进程失败: {}", e),
                        }
                    }
                    
                    // 2. 检查并清理带有数字后缀的 Metatrader5-*.exe 进程
                    // 使用通配符查找所有Metatrader5-*.exe进程
                    let output = std::process::Command::new("wmic")
                        .args(&["process", "where", "name like 'Metatrader5-%.exe'", "get", "name"])
                        .output()?;
                        
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.contains("Metatrader5-") {
                        tracing::warn!("发现Metatrader5-*.exe进程, 正在清理...");
                        
                        // 使用任务管理器的筛选功能清理所有匹配的进程
                        let kill_result = std::process::Command::new("taskkill")
                            .args(&["/F", "/IM", "Metatrader5-*.exe"])
                            .output();
                            
                        match kill_result {
                            Ok(_) => tracing::info!("成功清理 Metatrader5-*.exe 进程"),
                            Err(e) => tracing::warn!("清理 Metatrader5-*.exe 进程失败: {}", e),
                        }
                    }
                    
                    // 3. 如果上面的通配符方法不起作用，可以尝试列出所有进程并逐一匹配
                    let output = std::process::Command::new("tasklist")
                        .args(&["/FO", "CSV"])
                        .output()?;
                        
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    let lines: Vec<&str> = output_str.lines().collect();
                    
                    for line in lines {
                        if line.contains("Metatrader5-") {
                            // 从行中提取进程名称
                            if let Some(process_name) = line.split(',').nth(0) {
                                let process_name = process_name.trim_matches('"');
                                tracing::warn!("发现MetaTrader5相关进程: {}, 正在清理...", process_name);
                                
                                let kill_result = std::process::Command::new("taskkill")
                                    .args(&["/F", "/IM", process_name])
                                    .output();
                                    
                                match kill_result {
                                    Ok(_) => tracing::info!("成功清理进程: {}", process_name),
                                    Err(e) => tracing::warn!("清理进程 {} 失败: {}", process_name, e),
                                }
                            }
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


// async fn bind_with_retry(addr: SocketAddr, max_retries: u32) -> Result<tokio::net::TcpListener, Box<dyn std::error::Error>> {
//     let mut retries = 0;
//     loop {
//         match tokio::net::TcpListener::bind(addr).await {
//             Ok(listener) => return Ok(listener),
//             Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
//                 if retries >= max_retries {
//                     return Err(format!("端口 {} 被占用，重试 {} 次后仍然失败", addr.port(), max_retries).into());
//                 }
//                 tracing::warn!("端口 {} 被占用，尝试清理 MetaTrader5 进程...", addr.port());
                
//                 #[cfg(windows)]
//                 {
//                     // 查找并清理 MetaTrader5 进程
//                     let output = std::process::Command::new("tasklist")
//                         .args(&["/FI", "IMAGENAME eq MetaTrader5.exe", "/FO", "CSV"])
//                         .output()?;
                    
//                     let output_str = String::from_utf8_lossy(&output.stdout);
//                     if output_str.contains("MetaTrader5.exe") {
//                         tracing::warn!("发现旧的MetaTrader5进程, 正在清理...");
                        
//                         // 强制结束所有MetaTrader5.exe进程
//                         let kill_result = std::process::Command::new("taskkill")
//                             .args(&["/F", "/IM", "MetaTrader5.exe"])
//                             .output();
                            
//                         match kill_result {
//                             Ok(_) => tracing::info!("成功清理 MetaTrader5 进程"),
//                             Err(e) => tracing::warn!("清理 MetaTrader5 进程失败: {}", e),
//                         }
//                     }
//                 }
                
//                 // 等待进程完全退出
//                 tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
//                 retries += 1;
//             }
//             Err(e) => return Err(e.into()),
//         }
//     }
// }






